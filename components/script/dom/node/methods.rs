/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::LazyCell;
use std::cmp;
use std::slice::from_ref;

use js::context::JSContext;
use script_bindings::codegen::InheritTypes::DocumentFragmentTypeId;
use smallvec::SmallVec;
use xml5ever::ns;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{
    GetRootNodeOptions, NodeConstants, NodeMethods,
};
use crate::dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use crate::dom::bindings::domname::namespace_from_domstring;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId, TextTypeId};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::Element;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::mutationobserver::{Mutation, MutationObserver};
use crate::dom::node::nodelist::NodeList;
use crate::dom::node::{SuppressObserver, as_uintptr};
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::text::Text;
use crate::dom::types::CDATASection;
use crate::dom::virtualmethods::vtable_for;
use crate::dom::{ChildrenMutation, CloneChildrenFlag, Node, NodeTraits};
use crate::script_runtime::CanGc;

impl NodeMethods<crate::DomTypeHolder> for Node {
    /// <https://dom.spec.whatwg.org/#dom-node-nodetype>
    fn NodeType(&self) -> u16 {
        match self.type_id() {
            NodeTypeId::Attr => NodeConstants::ATTRIBUTE_NODE,
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                NodeConstants::TEXT_NODE
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::CDATASection)) => {
                NodeConstants::CDATA_SECTION_NODE
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                NodeConstants::PROCESSING_INSTRUCTION_NODE
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => NodeConstants::COMMENT_NODE,
            NodeTypeId::Document(_) => NodeConstants::DOCUMENT_NODE,
            NodeTypeId::DocumentType => NodeConstants::DOCUMENT_TYPE_NODE,
            NodeTypeId::DocumentFragment(_) => NodeConstants::DOCUMENT_FRAGMENT_NODE,
            NodeTypeId::Element(_) => NodeConstants::ELEMENT_NODE,
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nodename>
    fn NodeName(&self) -> DOMString {
        match self.type_id() {
            NodeTypeId::Attr => self.downcast::<Attr>().unwrap().qualified_name(),
            NodeTypeId::Element(..) => self.downcast::<Element>().unwrap().TagName(),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                DOMString::from("#text")
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::CDATASection)) => {
                DOMString::from("#cdata-section")
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                self.downcast::<ProcessingInstruction>().unwrap().Target()
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => DOMString::from("#comment"),
            NodeTypeId::DocumentType => self.downcast::<DocumentType>().unwrap().name().clone(),
            NodeTypeId::DocumentFragment(_) => DOMString::from("#document-fragment"),
            NodeTypeId::Document(_) => DOMString::from("#document"),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-baseuri>
    fn BaseURI(&self) -> USVString {
        USVString(String::from(self.owner_doc().base_url().as_str()))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-isconnected>
    fn IsConnected(&self) -> bool {
        self.is_connected()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-ownerdocument>
    fn GetOwnerDocument(&self) -> Option<DomRoot<Document>> {
        match self.type_id() {
            NodeTypeId::Document(_) => None,
            _ => Some(self.owner_doc()),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-getrootnode>
    fn GetRootNode(&self, options: &GetRootNodeOptions) -> DomRoot<Node> {
        if !options.composed &&
            let Some(shadow_root) = self.containing_shadow_root()
        {
            return DomRoot::upcast(shadow_root);
        }

        if self.is_connected() {
            DomRoot::from_ref(self.owner_doc().upcast::<Node>())
        } else {
            self.inclusive_ancestors(ShadowIncluding::Yes)
                .last()
                .unwrap()
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-parentnode>
    fn GetParentNode(&self) -> Option<DomRoot<Node>> {
        self.parent_node().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-parentelement>
    fn GetParentElement(&self) -> Option<DomRoot<Element>> {
        self.GetParentNode().and_then(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-haschildnodes>
    fn HasChildNodes(&self) -> bool {
        self.first_child().get().is_some()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-childnodes>
    fn ChildNodes(&self, cx: &mut JSContext) -> DomRoot<NodeList> {
        if let Some(list) = self.ensure_rare_data().child_list.get() {
            return list;
        }

        let doc = self.owner_doc();
        let window = doc.window();
        let list = NodeList::new_child_list(window, self, CanGc::from_cx(cx));
        self.ensure_rare_data().child_list.set(Some(&list));
        list
    }

    /// <https://dom.spec.whatwg.org/#dom-node-firstchild>
    fn GetFirstChild(&self) -> Option<DomRoot<Node>> {
        self.first_child().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-lastchild>
    fn GetLastChild(&self) -> Option<DomRoot<Node>> {
        self.last_child().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-previoussibling>
    fn GetPreviousSibling(&self) -> Option<DomRoot<Node>> {
        self.prev_sibling().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nextsibling>
    fn GetNextSibling(&self) -> Option<DomRoot<Node>> {
        self.next_sibling().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nodevalue>
    fn GetNodeValue(&self) -> Option<DOMString> {
        match self.type_id() {
            NodeTypeId::Attr => Some(self.downcast::<Attr>().unwrap().Value()),
            NodeTypeId::CharacterData(_) => {
                self.downcast::<CharacterData>().map(CharacterData::Data)
            },
            _ => None,
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nodevalue>
    fn SetNodeValue(&self, cx: &mut JSContext, val: Option<DOMString>) -> Fallible<()> {
        match self.type_id() {
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(cx, val.unwrap_or_default())?;
            },
            NodeTypeId::CharacterData(_) => {
                let character_data = self.downcast::<CharacterData>().unwrap();
                character_data.SetData(cx, val.unwrap_or_default());
            },
            _ => {},
        };
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-node-textcontent>
    fn GetTextContent(&self) -> Option<DOMString> {
        match self.type_id() {
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                let content =
                    Node::collect_text_contents(self.traverse_preorder(ShadowIncluding::No));
                Some(content)
            },
            NodeTypeId::Attr => Some(self.downcast::<Attr>().unwrap().Value()),
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                Some(characterdata.Data())
            },
            NodeTypeId::DocumentType | NodeTypeId::Document(_) => None,
        }
    }

    /// <https://dom.spec.whatwg.org/#set-text-content>
    fn SetTextContent(&self, cx: &mut JSContext, value: Option<DOMString>) -> Fallible<()> {
        match self.type_id() {
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                self.set_text_content_for_element(cx, value);
            },
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(cx, value.unwrap_or_default())?;
            },
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                characterdata.SetData(cx, value.unwrap_or_default());
            },
            NodeTypeId::DocumentType | NodeTypeId::Document(_) => {},
        };
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-node-insertbefore>
    fn InsertBefore(
        &self,
        cx: &mut JSContext,
        node: &Node,
        child: Option<&Node>,
    ) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(cx, node, self, child)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-appendchild>
    fn AppendChild(&self, cx: &mut JSContext, node: &Node) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(cx, node, self, None)
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace>
    fn ReplaceChild(
        &self,
        cx: &mut JSContext,
        node: &Node,
        child: &Node,
    ) -> Fallible<DomRoot<Node>> {
        // Step 1. If parent is not a Document, DocumentFragment, or Element node,
        // then throw a "HierarchyRequestError" DOMException.
        match self.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
            },
            _ => return Err(Error::HierarchyRequest(None)),
        }

        // Step 2. If node is a host-including inclusive ancestor of parent,
        // then throw a "HierarchyRequestError" DOMException.
        if node.is_inclusive_ancestor_of(self) {
            return Err(Error::HierarchyRequest(None));
        }

        // Step 3. If child’s parent is not parent, then throw a "NotFoundError" DOMException.
        if !self.is_parent_of(child) {
            return Err(Error::NotFound(None));
        }

        // Step 4. If node is not a DocumentFragment, DocumentType, Element, or CharacterData node,
        // then throw a "HierarchyRequestError" DOMException.
        // Step 5. If either node is a Text node and parent is a document,
        // or node is a doctype and parent is not a document, then throw a "HierarchyRequestError" DOMException.
        match node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) if self.is::<Document>() => {
                return Err(Error::HierarchyRequest(None));
            },
            NodeTypeId::DocumentType if !self.is::<Document>() => {
                return Err(Error::HierarchyRequest(None));
            },
            NodeTypeId::Document(_) | NodeTypeId::Attr => {
                return Err(Error::HierarchyRequest(None));
            },
            _ => (),
        }

        // Step 6. If parent is a document, and any of the statements below, switched on the interface node implements,
        // are true, then throw a "HierarchyRequestError" DOMException.
        if self.is::<Document>() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment(_) => {
                    // Step 6.1.1(b)
                    if node.children_unrooted(cx.no_gc()).any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match node.child_elements_unrooted(cx.no_gc()).count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if self
                                .child_elements_unrooted(cx.no_gc())
                                .any(|c| c.upcast::<Node>() != child)
                            {
                                return Err(Error::HierarchyRequest(None));
                            }
                            if child.following_siblings().any(|child| child.is_doctype()) {
                                return Err(Error::HierarchyRequest(None));
                            }
                        },
                        // Step 6.1.1(a)
                        _ => return Err(Error::HierarchyRequest(None)),
                    }
                },
                // Step 6.2
                NodeTypeId::Element(..) => {
                    if self
                        .child_elements_unrooted(cx.no_gc())
                        .any(|c| c.upcast::<Node>() != child)
                    {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if child.following_siblings().any(|child| child.is_doctype()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if self
                        .children_unrooted(cx.no_gc())
                        .any(|c| c.is_doctype() && *c != child)
                    {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if self
                        .children_unrooted(cx.no_gc())
                        .take_while(|c| **c != child)
                        .any(|c| c.is::<Element>())
                    {
                        return Err(Error::HierarchyRequest(None));
                    }
                },
                NodeTypeId::CharacterData(..) => (),
                // Because Document and Attr should already throw `HierarchyRequest`
                // error, both of them are unreachable here.
                NodeTypeId::Document(_) => unreachable!(),
                NodeTypeId::Attr => unreachable!(),
            }
        }

        // Step 7. Let referenceChild be child’s next sibling.
        // Step 8. If referenceChild is node, then set referenceChild to node’s next sibling.
        let child_next_sibling = child.GetNextSibling();
        let node_next_sibling = node.GetNextSibling();
        let reference_child = if child_next_sibling.as_deref() == Some(node) {
            node_next_sibling.as_deref()
        } else {
            child_next_sibling.as_deref()
        };

        // Step 9. Let previousSibling be child’s previous sibling.
        let previous_sibling = child.GetPreviousSibling();

        // NOTE: All existing browsers assume that adoption is performed here, which does not follow the DOM spec.
        // However, if we follow the spec and delay adoption to inside `Node::insert()`, then the mutation records will
        // be different, and we will fail WPT dom/nodes/MutationObserver-childList.html.
        let document = self.owner_document();
        Node::adopt(cx, node, &document);

        // Step 10. Let removedNodes be the empty set.
        // Step 11. If child’s parent is non-null:
        //     1. Set removedNodes to « child ».
        //     2. Remove child with the suppress observers flag set.
        let removed_child = if node != child {
            // Step 11.
            Node::remove(cx, child, self, SuppressObserver::Suppressed);
            Some(child)
        } else {
            None
        };

        // Step 12. Let nodes be node’s children if node is a DocumentFragment node; otherwise « node ».
        rooted_vec!(let mut nodes);
        let nodes = if node.type_id() ==
            NodeTypeId::DocumentFragment(DocumentFragmentTypeId::DocumentFragment) ||
            node.type_id() == NodeTypeId::DocumentFragment(DocumentFragmentTypeId::ShadowRoot)
        {
            nodes.extend(node.children().map(|node| Dom::from_ref(&*node)));
            nodes.r()
        } else {
            from_ref(&node)
        };

        // Step 13. Insert node into parent before referenceChild with the suppress observers flag set.
        Node::insert(
            cx,
            node,
            self,
            reference_child,
            SuppressObserver::Suppressed,
        );

        vtable_for(self).children_changed(
            cx,
            &ChildrenMutation::replace(
                previous_sibling.as_deref(),
                &removed_child,
                nodes,
                reference_child,
            ),
        );

        // Step 14. Queue a tree mutation record for parent with nodes, removedNodes,
        // previousSibling, and referenceChild.
        let removed = removed_child.map(|r| [r]);
        let mutation = LazyCell::new(|| Mutation::ChildList {
            added: Some(nodes),
            removed: removed.as_ref().map(|r| &r[..]),
            prev: previous_sibling.as_deref(),
            next: reference_child,
        });

        MutationObserver::queue_a_mutation_record(self, mutation);

        // Step 15. Return child.
        Ok(DomRoot::from_ref(child))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-removechild>
    fn RemoveChild(&self, cx: &mut JSContext, node: &Node) -> Fallible<DomRoot<Node>> {
        Node::pre_remove(cx, node, self)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-normalize>
    fn Normalize(&self, cx: &mut JSContext) {
        let mut children = self.children().enumerate().peekable();
        while let Some((_, node)) = children.next() {
            if let Some(text) = node.downcast::<Text>() {
                if text.is::<CDATASection>() {
                    continue;
                }
                let cdata = text.upcast::<CharacterData>();
                let mut length = cdata.Length();
                if length == 0 {
                    Node::remove(cx, &node, self, SuppressObserver::Unsuppressed);
                    continue;
                }
                while children.peek().is_some_and(|(_, sibling)| {
                    sibling.is::<Text>() && !sibling.is::<CDATASection>()
                }) {
                    let (index, sibling) = children.next().unwrap();
                    sibling
                        .ranges()
                        .drain_to_preceding_text_sibling(&sibling, &node, length);
                    self.ranges()
                        .move_to_text_child_at(self, index as u32, &node, length);
                    let sibling_cdata = sibling.downcast::<CharacterData>().unwrap();
                    length += sibling_cdata.Length();
                    cdata.append_data(cx, &sibling_cdata.data());
                    Node::remove(cx, &sibling, self, SuppressObserver::Unsuppressed);
                }
            } else {
                node.Normalize(cx);
            }
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-clonenode>
    fn CloneNode(&self, cx: &mut JSContext, subtree: bool) -> Fallible<DomRoot<Node>> {
        // Step 1. If this is a shadow root, then throw a "NotSupportedError" DOMException.
        if self.is::<ShadowRoot>() {
            return Err(Error::NotSupported(None));
        }

        // Step 2. Return the result of cloning a node given this with subtree set to subtree.
        let result = Node::clone(
            cx,
            self,
            None,
            if subtree {
                CloneChildrenFlag::CloneChildren
            } else {
                CloneChildrenFlag::DoNotCloneChildren
            },
            None,
        );
        Ok(result)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-isequalnode>
    fn IsEqualNode(&self, maybe_node: Option<&Node>) -> bool {
        fn is_equal_doctype(node: &Node, other: &Node) -> bool {
            let doctype = node.downcast::<DocumentType>().unwrap();
            let other_doctype = other.downcast::<DocumentType>().unwrap();
            (*doctype.name() == *other_doctype.name()) &&
                (*doctype.public_id() == *other_doctype.public_id()) &&
                (*doctype.system_id() == *other_doctype.system_id())
        }
        fn is_equal_element(node: &Node, other: &Node) -> bool {
            let element = node.downcast::<Element>().unwrap();
            let other_element = other.downcast::<Element>().unwrap();
            (*element.namespace() == *other_element.namespace()) &&
                (*element.prefix() == *other_element.prefix()) &&
                (*element.local_name() == *other_element.local_name()) &&
                (element.attrs().borrow().len() == other_element.attrs().borrow().len())
        }
        fn is_equal_processinginstruction(node: &Node, other: &Node) -> bool {
            let pi = node.downcast::<ProcessingInstruction>().unwrap();
            let other_pi = other.downcast::<ProcessingInstruction>().unwrap();
            (*pi.target() == *other_pi.target()) &&
                (*pi.upcast::<CharacterData>().data() ==
                    *other_pi.upcast::<CharacterData>().data())
        }
        fn is_equal_characterdata(node: &Node, other: &Node) -> bool {
            let characterdata = node.downcast::<CharacterData>().unwrap();
            let other_characterdata = other.downcast::<CharacterData>().unwrap();
            *characterdata.data() == *other_characterdata.data()
        }
        fn is_equal_attr(node: &Node, other: &Node) -> bool {
            let attr = node.downcast::<Attr>().unwrap();
            let other_attr = other.downcast::<Attr>().unwrap();
            (*attr.namespace() == *other_attr.namespace()) &&
                (attr.local_name() == other_attr.local_name()) &&
                (**attr.value() == **other_attr.value())
        }
        fn is_equal_element_attrs(node: &Node, other: &Node) -> bool {
            let element = node.downcast::<Element>().unwrap();
            let other_element = other.downcast::<Element>().unwrap();
            assert!(element.attrs().borrow().len() == other_element.attrs().borrow().len());
            element.attrs().borrow().iter().all(|attr| {
                other_element.attrs().borrow().iter().any(|other_attr| {
                    (*attr.namespace() == *other_attr.namespace()) &&
                        (attr.local_name() == other_attr.local_name()) &&
                        (**attr.value() == **other_attr.value())
                })
            })
        }

        fn is_equal_node(this: &Node, node: &Node) -> bool {
            // Step 2.
            if this.NodeType() != node.NodeType() {
                return false;
            }

            match node.type_id() {
                // Step 3.
                NodeTypeId::DocumentType if !is_equal_doctype(this, node) => return false,
                NodeTypeId::Element(..) if !is_equal_element(this, node) => return false,
                NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction)
                    if !is_equal_processinginstruction(this, node) =>
                {
                    return false;
                },
                NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) |
                NodeTypeId::CharacterData(CharacterDataTypeId::Comment)
                    if !is_equal_characterdata(this, node) =>
                {
                    return false;
                },
                // Step 4.
                NodeTypeId::Element(..) if !is_equal_element_attrs(this, node) => return false,
                NodeTypeId::Attr if !is_equal_attr(this, node) => return false,

                _ => (),
            }

            // Step 5.
            if this.children_count() != node.children_count() {
                return false;
            }

            // Step 6.
            this.children()
                .zip(node.children())
                .all(|(child, other_child)| is_equal_node(&child, &other_child))
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(node) => is_equal_node(self, node),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-issamenode>
    fn IsSameNode(&self, other_node: Option<&Node>) -> bool {
        match other_node {
            Some(node) => self == node,
            None => false,
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-comparedocumentposition>
    fn CompareDocumentPosition(&self, other: &Node) -> u16 {
        // Step 1. If this is other, then return zero.
        if self == other {
            return 0;
        }

        // Step 2. Let node1 be other and node2 be this.
        let mut node1 = Some(other);
        let mut node2 = Some(self);

        // Step 3. Let attr1 and attr2 be null.
        let mut attr1: Option<&Attr> = None;
        let mut attr2: Option<&Attr> = None;

        // step 4: spec says to operate on node1 here,
        // node1 is definitely Some(other) going into this step
        // The compiler doesn't know the lifetime of attr1.GetOwnerElement
        // is guaranteed by the lifetime of attr1, so we hold it explicitly
        let attr1owner;
        if let Some(a) = other.downcast::<Attr>() {
            attr1 = Some(a);
            attr1owner = a.GetOwnerElement();
            node1 = match attr1owner {
                Some(ref e) => Some(e.upcast()),
                None => None,
            }
        }

        // step 5.1: spec says to operate on node2 here,
        // node2 is definitely just Some(self) going into this step
        let attr2owner;
        if let Some(a) = self.downcast::<Attr>() {
            attr2 = Some(a);
            attr2owner = a.GetOwnerElement();
            node2 = match attr2owner {
                Some(ref e) => Some(e.upcast()),
                None => None,
            }
        }

        // Step 5.2
        // This substep seems lacking in test coverage.
        // We hit this when comparing two attributes that have the
        // same owner element.
        if let Some(node2) = node2 &&
            Some(node2) == node1 &&
            let (Some(a1), Some(a2)) = (attr1, attr2)
        {
            let attrs = node2.downcast::<Element>().unwrap().attrs();
            // go through the attrs in order to see if self
            // or other is first; spec is clear that we
            // want value-equality, not reference-equality
            for attr in attrs.borrow().iter() {
                if (*attr.namespace() == *a1.namespace()) &&
                    (attr.local_name() == a1.local_name()) &&
                    (**attr.value() == **a1.value())
                {
                    return NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC +
                        NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                if (*attr.namespace() == *a2.namespace()) &&
                    (attr.local_name() == a2.local_name()) &&
                    (**attr.value() == **a2.value())
                {
                    return NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC +
                        NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
            }
            // both attrs have node2 as their owner element, so
            // we can't have left the loop without seeing them
            unreachable!();
        }

        // Step 6
        match (node1, node2) {
            (None, _) => {
                // node1 is null
                NodeConstants::DOCUMENT_POSITION_FOLLOWING +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
            },
            (_, None) => {
                // node2 is null
                NodeConstants::DOCUMENT_POSITION_PRECEDING +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
            },
            (Some(node1), Some(node2)) => {
                // still step 6, testing if node1 and 2 share a root
                let mut self_and_ancestors = node2
                    .inclusive_ancestors(ShadowIncluding::No)
                    .collect::<SmallVec<[_; 20]>>();
                let mut other_and_ancestors = node1
                    .inclusive_ancestors(ShadowIncluding::No)
                    .collect::<SmallVec<[_; 20]>>();

                if self_and_ancestors.last() != other_and_ancestors.last() {
                    let random = as_uintptr(self_and_ancestors.last().unwrap()) <
                        as_uintptr(other_and_ancestors.last().unwrap());
                    let random = if random {
                        NodeConstants::DOCUMENT_POSITION_FOLLOWING
                    } else {
                        NodeConstants::DOCUMENT_POSITION_PRECEDING
                    };

                    // Disconnected.
                    return random +
                        NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                        NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC;
                }
                // steps 7-10
                let mut parent = self_and_ancestors.pop().unwrap();
                other_and_ancestors.pop().unwrap();

                let mut current_position =
                    cmp::min(self_and_ancestors.len(), other_and_ancestors.len());

                while current_position > 0 {
                    current_position -= 1;
                    let child_1 = self_and_ancestors.pop().unwrap();
                    let child_2 = other_and_ancestors.pop().unwrap();

                    if child_1 != child_2 {
                        for child in parent.children() {
                            if child == child_1 {
                                // `other` is following `self`.
                                return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                            }
                            if child == child_2 {
                                // `other` is preceding `self`.
                                return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                            }
                        }
                    }

                    parent = child_1;
                }

                // We hit the end of one of the parent chains, so one node needs to be
                // contained in the other.
                //
                // If we're the container, return that `other` is contained by us.
                if self_and_ancestors.len() < other_and_ancestors.len() {
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING +
                        NodeConstants::DOCUMENT_POSITION_CONTAINED_BY
                } else {
                    NodeConstants::DOCUMENT_POSITION_PRECEDING +
                        NodeConstants::DOCUMENT_POSITION_CONTAINS
                }
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-contains>
    fn Contains(&self, maybe_other: Option<&Node>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-lookupprefix>
    fn LookupPrefix(&self, namespace: Option<DOMString>) -> Option<DOMString> {
        let namespace = namespace_from_domstring(namespace);

        // Step 1.
        if namespace == ns!() {
            return None;
        }

        // Step 2.
        match self.type_id() {
            NodeTypeId::Element(..) => self.downcast::<Element>().unwrap().lookup_prefix(namespace),
            NodeTypeId::Document(_) => self
                .downcast::<Document>()
                .unwrap()
                .GetDocumentElement()
                .and_then(|element| element.lookup_prefix(namespace)),
            NodeTypeId::DocumentType | NodeTypeId::DocumentFragment(_) => None,
            NodeTypeId::Attr => self
                .downcast::<Attr>()
                .unwrap()
                .GetOwnerElement()
                .and_then(|element| element.lookup_prefix(namespace)),
            _ => self
                .GetParentElement()
                .and_then(|element| element.lookup_prefix(namespace)),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri>
    fn LookupNamespaceURI(&self, prefix: Option<DOMString>) -> Option<DOMString> {
        // Step 1. If prefix is the empty string, then set it to null.
        let prefix = prefix.filter(|prefix| !prefix.is_empty());

        // Step 2. Return the result of running locate a namespace for this using prefix.
        Node::namespace_to_string(Node::locate_namespace(self, prefix))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-isdefaultnamespace>
    fn IsDefaultNamespace(&self, namespace: Option<DOMString>) -> bool {
        // Step 1.
        let namespace = namespace_from_domstring(namespace);
        // Steps 2 and 3.
        Node::locate_namespace(self, None) == namespace
    }
}
