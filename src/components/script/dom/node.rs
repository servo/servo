/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, ElementCast, TextCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, NodeBase, NodeDerived};
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object2};
use dom::bindings::utils::{DOMString, null_str_as_empty};
use dom::bindings::utils::{ErrorResult, Fallible, NotFound, HierarchyRequest};
use dom::characterdata::CharacterData;
use dom::document::{Document, DocumentTypeId};
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementTypeId, HTMLAnchorElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::nodelist::{NodeList};
use dom::text::Text;

use js::jsapi::{JSObject, JSContext};

use layout_interface::{LayoutChan, ReapLayoutDataMsg};

use std::cast;
use std::cast::transmute;
use std::cell::{RefCell, Ref, RefMut};
use std::iter::{Map, Filter};
use std::util;

//
// The basic Node structure
//

/// An HTML node.
pub struct Node {
    /// The JavaScript reflector for this node.
    eventtarget: EventTarget,

    /// The type of node that this is.
    type_id: NodeTypeId,

    /// The parent of this node.
    parent_node: Option<JSManaged<Node>>,

    /// The first child of this node.
    first_child: Option<JSManaged<Node>>,

    /// The last child of this node.
    last_child: Option<JSManaged<Node>>,

    /// The next sibling of this node.
    next_sibling: Option<JSManaged<Node>>,

    /// The previous sibling of this node.
    prev_sibling: Option<JSManaged<Node>>,

    /// The document that this node belongs to.
    priv owner_doc: Option<JSManaged<Document>>,

    /// The live list of children return by .childNodes.
    child_list: Option<@mut NodeList>,

    /// A bitfield of flags for node items.
    priv flags: NodeFlags,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// FIXME(pcwalton): We need to send these back to the layout task to be destroyed when this
    /// node is finalized.
    layout_data: LayoutDataRef,
}

impl NodeDerived for EventTarget {
    fn is_node(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(_) => true,
            _ => false
        }
    }
}

/// Flags for node items.
pub struct NodeFlags(u8);

impl NodeFlags {
    pub fn new(type_id: NodeTypeId) -> NodeFlags {
        let mut flags = NodeFlags(0);
        match type_id {
            DocumentNodeTypeId(_) => { flags.set_is_in_doc(true); }
            _ => {}
        }
        flags
    }
}

/// Specifies whether this node is in a document.
bitfield!(NodeFlags, is_in_doc, set_is_in_doc, 0x01)

#[unsafe_destructor]
impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            let this: &mut Node = cast::transmute(self);
            this.reap_layout_data()
        }
    }
}

/// Encapsulates the abstract layout data.
pub struct LayoutData {
    priv chan: Option<LayoutChan>,
    priv data: *(),
}

pub struct LayoutDataRef {
    data_cell: RefCell<Option<LayoutData>>,
}

impl LayoutDataRef {
    pub fn new() -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(None),
        }
    }

    pub unsafe fn from_data<T>(data: ~T) -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(Some(cast::transmute(data))),
        }
    }

    /// Returns true if there is layout data present.
    #[inline]
    pub fn is_present(&self) -> bool {
        let data_ref = self.data_cell.borrow();
        data_ref.get().is_some()
    }

    /// Take the chan out of the layout data if it is present.
    pub fn take_chan(&self) -> Option<LayoutChan> {
        let mut data_ref = self.data_cell.borrow_mut();
        let layout_data = data_ref.get();
        match *layout_data {
            None => None,
            Some(..) => Some(layout_data.get_mut_ref().chan.take_unwrap()),
        }
    }

    /// Borrows the layout data immutably, *asserting that there are no mutators*. Bad things will
    /// happen if you try to mutate the layout data while this is held. This is the only thread-
    /// safe layout data accessor.
    ///
    /// FIXME(pcwalton): Enforce this invariant via the type system. Will require traversal
    /// functions to be trusted, but c'est la vie.
    // #[inline]
    // pub unsafe fn borrow_unchecked<'a>(&'a self) -> &'a () {
    //     self.data.borrow_unchecked()
    // }

    /// Borrows the layout data immutably. This function is *not* thread-safe.
    #[inline]
    pub fn borrow<'a>(&'a self) -> Ref<'a,Option<LayoutData>> {
        self.data_cell.borrow()
    }

    /// Borrows the layout data mutably. This function is *not* thread-safe.
    ///
    /// FIXME(pcwalton): We should really put this behind a `MutLayoutView` phantom type, to
    /// prevent CSS selector matching from mutably accessing nodes it's not supposed to and racing
    /// on it. This has already resulted in one bug!
    #[inline]
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a,Option<LayoutData>> {
        self.data_cell.borrow_mut()
    }
}

/// A trait that represents abstract layout data.
/// 
/// FIXME(pcwalton): Very very unsafe!!! We need to send these back to the layout task to be
/// destroyed when this node is finalized.
pub trait TLayoutData {}

/// The different types of nodes.
#[deriving(Eq)]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    DocumentFragmentNodeTypeId,
    CommentNodeTypeId,
    DocumentNodeTypeId(DocumentTypeId),
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
}

pub trait INode {
    fn AppendChild(mut self, node: JSManaged<Node>) -> Fallible<JSManaged<Node>>;
    fn ReplaceChild(mut self, node: JSManaged<Node>, child: JSManaged<Node>) -> Fallible<JSManaged<Node>>;
    fn RemoveChild(mut self, node: JSManaged<Node>) -> Fallible<JSManaged<Node>>;
}

impl INode for JSManaged<Node> {
    fn AppendChild(mut self, node: JSManaged<Node>) -> Fallible<JSManaged<Node>> {
        self.mut_value().AppendChild(self, node)
    }

    fn ReplaceChild(mut self, node: JSManaged<Node>, child: JSManaged<Node>) -> Fallible<JSManaged<Node>> {
        self.mut_value().ReplaceChild(node, child)
    }

    fn RemoveChild(mut self, node: JSManaged<Node>) -> Fallible<JSManaged<Node>> {
        self.mut_value().RemoveChild(self, node)
    }
}

pub trait NodeHelpers {
    fn ancestors(&self) -> AncestorIterator;
    fn children(&self) -> AbstractNodeChildrenIterator;
    fn child_elements(&self) -> ChildElementIterator;
    fn is_in_doc(&self) -> bool;

    fn type_id(self) -> NodeTypeId;

    fn parent_node(&self) -> Option<JSManaged<Node>>;
    fn first_child(&self) -> Option<JSManaged<Node>>;
    fn last_child(&self) -> Option<JSManaged<Node>>;
    fn prev_sibling(self) -> Option<JSManaged<Node>>;
    fn next_sibling(self) -> Option<JSManaged<Node>>;

    fn is_element(&self) -> bool;
    fn is_document(&self) -> bool;
    fn is_doctype(self) -> bool;
    fn is_text(self) -> bool;
    fn is_anchor_element(self) -> bool;

    fn node_inserted(self);
    fn node_removed(self);
    fn add_child(mut self, new_child: JSManaged<Node>, before: Option<JSManaged<Node>>);
    fn remove_child(mut self, child: JSManaged<Node>);

    fn dump(&self);
    fn dump_indent(&self, indent: uint);
    fn debug_str(&self) -> ~str;

    fn traverse_preorder(&self) -> TreeIterator;
    fn sequential_traverse_postorder(&self) -> TreeIterator;
}

impl NodeHelpers for JSManaged<Node> {
    /// Dumps the subtree rooted at this node, for debugging.
    fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for _ in range(0, indent) {
            s.push_str("    ");
        }

        s.push_str(self.debug_str());
        debug!("{:s}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1u)
        }
    }

    /// Returns a string that describes this node.
    fn debug_str(&self) -> ~str {
        format!("{:?}", self.type_id())
    }

    /// Iterates over all ancestors of this node.
    fn ancestors(&self) -> AncestorIterator {
        self.value().ancestors()
    }

    fn children(&self) -> AbstractNodeChildrenIterator {
        self.value().children()
    }

    fn child_elements(&self) -> ChildElementIterator {
        self.value().child_elements()
    }

    fn is_in_doc(&self) -> bool {
        self.value().flags.is_in_doc()
    }

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(self) -> NodeTypeId {
        self.value().type_id
    }

    fn parent_node(&self) -> Option<JSManaged<Node>> {
        self.value().parent_node
    }

    fn first_child(&self) -> Option<JSManaged<Node>> {
        self.value().first_child
    }

    fn last_child(&self) -> Option<JSManaged<Node>> {
        self.value().last_child
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    fn prev_sibling(self) -> Option<JSManaged<Node>> {
        self.value().prev_sibling
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    fn next_sibling(self) -> Option<JSManaged<Node>> {
        self.value().next_sibling
    }

    #[inline]
    fn is_element(&self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(..) => true,
            _ => false
        }
    }

    #[inline]
    fn is_document(&self) -> bool {
        match self.type_id() {
            DocumentNodeTypeId(..) => true,
            _ => false
        }
    }

    #[inline]
    fn is_anchor_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLAnchorElementTypeId)
    }

    #[inline]
    fn is_doctype(self) -> bool {
        self.type_id() == DoctypeNodeTypeId
    }

    #[inline]
    fn is_text(self) -> bool {
        // FIXME(pcwalton): Temporary workaround for the lack of inlining of autogenerated `Eq`
        // implementations in Rust.
        self.type_id() == TextNodeTypeId
    }

    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(self) {
        assert!(self.parent_node().is_some());
        let mut document = self.value().owner_doc();

        // Register elements having "id" attribute to the owner doc.
        if self.is_element() {
            document.mut_value().register_nodes_with_id(&ElementCast::to(self));
        }

        document.value().content_changed();
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(self) {
        assert!(self.parent_node().is_none());
        let mut document = self.value().owner_doc();

        // Unregister elements having "id".
        if self.is_element() {
            document.mut_value().unregister_nodes_with_id(&ElementCast::to(self));
        }

        document.value().content_changed();
    }

    //
    // Pointer stitching
    //

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(mut self, mut new_child: JSManaged<Node>, before: Option<JSManaged<Node>>) {
        //let this_node = self.mut_value();
        //let new_child_node = new_child.mut_value();
        assert!(new_child.parent_node().is_none());
        assert!(new_child.prev_sibling().is_none());
        assert!(new_child.next_sibling().is_none());
        match before {
            Some(mut before) => {
                //let before_node = before.mut_value();
                // XXX Should assert that parent is self.
                assert!(before.parent_node().is_some());
                before.mut_value().set_prev_sibling(Some(new_child.clone()));
                new_child.mut_value().set_next_sibling(Some(before.clone()));
                match before.prev_sibling() {
                    None => {
                        // XXX Should assert that before is the first child of
                        //     self.
                        self.mut_value().set_first_child(Some(new_child.clone()));
                    },
                    Some(mut prev_sibling) => {
                        //let prev_sibling_node = prev_sibling.mut_value();
                        prev_sibling.mut_value().set_next_sibling(Some(new_child.clone()));
                        new_child.mut_value().set_prev_sibling(Some(prev_sibling.clone()));
                    },
                }
            },
            None => {
                match self.last_child() {
                    None => self.mut_value().set_first_child(Some(new_child.clone())),
                    Some(mut last_child) => {
                        //let last_child_node = last_child.mut_value();
                        assert!(last_child.next_sibling().is_none());
                        last_child.mut_value().set_next_sibling(Some(new_child.clone()));
                        new_child.mut_value().set_prev_sibling(Some(last_child.clone()));
                    }
                }

                self.mut_value().set_last_child(Some(new_child.clone()));
            },
        }

        new_child.mut_value().set_parent_node(Some(self.clone()));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node. (FIXME: This is not yet checked.)
    fn remove_child(mut self, mut child: JSManaged<Node>) {
        let this_node = self.mut_value();
        let child_node = child.mut_value();
        assert!(child_node.parent_node.is_some());

        match child_node.prev_sibling {
            None => this_node.set_first_child(child_node.next_sibling),
            Some(mut prev_sibling) => {
                let prev_sibling_node = prev_sibling.mut_value();
                prev_sibling_node.set_next_sibling(child_node.next_sibling);
            }
        }

        match child_node.next_sibling {
            None => this_node.set_last_child(child_node.prev_sibling),
            Some(mut next_sibling) => {
                let next_sibling_node = next_sibling.mut_value();
                next_sibling_node.set_prev_sibling(child_node.prev_sibling);
            }
        }

        child_node.set_prev_sibling(None);
        child_node.set_next_sibling(None);
        child_node.set_parent_node(None);
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(&self) -> TreeIterator {
        let mut nodes = ~[];
        gather_abstract_nodes(self, &mut nodes, false);
        TreeIterator::new(nodes)
    }

    /// Iterates over this node and all its descendants, in postorder.
    fn sequential_traverse_postorder(&self) -> TreeIterator {
        let mut nodes = ~[];
        gather_abstract_nodes(self, &mut nodes, true);
        TreeIterator::new(nodes)
    }
}

//
// Iteration and traversal
//

type ChildElementIterator<'a> = Map<'a, JSManaged<Node>,
                                JSManaged<Element>,
                                Filter<'a, JSManaged<Node>, AbstractNodeChildrenIterator>>;

pub struct AbstractNodeChildrenIterator {
    priv current_node: Option<JSManaged<Node>>,
}

impl Iterator<JSManaged<Node>> for AbstractNodeChildrenIterator {
    fn next(&mut self) -> Option<JSManaged<Node>> {
        let node = self.current_node;
        self.current_node = self.current_node.and_then(|node| {
            node.next_sibling()
        });
        node
    }
}

pub struct AncestorIterator {
    priv current: Option<JSManaged<Node>>,
}

impl Iterator<JSManaged<Node>> for AncestorIterator {
    fn next(&mut self) -> Option<JSManaged<Node>> {
        if self.current.is_none() {
            return None;
        }

        // FIXME: Do we need two clones here?
        let x = self.current.get_ref().clone();
        self.current = x.parent_node();
        Some(x.clone())
    }
}

// FIXME: Do this without precomputing a vector of refs.
// Easy for preorder; harder for postorder.
pub struct TreeIterator {
    priv nodes: ~[JSManaged<Node>],
    priv index: uint,
}

impl TreeIterator {
    fn new(nodes: ~[JSManaged<Node>]) -> TreeIterator {
        TreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl Iterator<JSManaged<Node>> for TreeIterator {
    fn next(&mut self) -> Option<JSManaged<Node>> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let v = self.nodes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

fn gather_abstract_nodes(cur: &JSManaged<Node>, refs: &mut ~[JSManaged<Node>], postorder: bool) {
    if !postorder {
        refs.push(cur.clone());
    }
    for kid in cur.children() {
        gather_abstract_nodes(&kid, refs, postorder)
    }
    if postorder {
        refs.push((*cur).clone());
    }
}

impl Node {
    pub fn ancestors(&self) -> AncestorIterator {
        AncestorIterator {
            current: self.parent_node,
        }
    }

    pub fn owner_doc(&self) -> JSManaged<Document> {
        self.owner_doc.unwrap()
    }

    pub fn set_owner_doc(&mut self, document: JSManaged<Document>) {
        self.owner_doc = Some(document);
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.first_child,
        }
    }

    pub fn child_elements(&self) -> ChildElementIterator {
        self.children().filter(|node| node.is_element())
                       .map(|node| {
                           let elem: JSManaged<Element> = ElementCast::to(node);
                           elem
                       })
    }

    pub fn reflect_node<N: Reflectable+NodeBase>
            (node:      ~N,
             document:  JSManaged<Document>,
             wrap_fn:   extern "Rust" fn(*JSContext, *JSObject, ~N) -> *JSObject)
             -> JSManaged<N> {
        assert!(node.reflector().get_jsobject().is_null());
        let node = reflect_dom_object2(node, document.value().window, wrap_fn);
        assert!(node.reflector().get_jsobject().is_not_null());
        node
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: JSManaged<Document>) -> Node {
        Node::new_(type_id, Some(doc))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<JSManaged<Document>>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(NodeTargetTypeId(type_id)),
            type_id: type_id,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            owner_doc: doc,
            child_list: None,

            flags: NodeFlags::new(type_id),

            layout_data: LayoutDataRef::new(),
        }
    }

    /// Sends layout data, if any, back to the script task to be destroyed.
    pub unsafe fn reap_layout_data(&mut self) {
        if self.layout_data.is_present() {
            let layout_data = util::replace(&mut self.layout_data, LayoutDataRef::new());
            let layout_chan = layout_data.take_chan();
            match layout_chan {
                None => {}
                Some(chan) => chan.send(ReapLayoutDataMsg(layout_data)),
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodetype
    pub fn NodeType(&self) -> u16 {
        match self.type_id {
            ElementNodeTypeId(_) => 1,
            TextNodeTypeId       => 3,
            CommentNodeTypeId    => 8,
            DocumentNodeTypeId(_)=> 9,
            DoctypeNodeTypeId    => 10,
            DocumentFragmentNodeTypeId => 11,
        }
    }

    pub fn NodeName(&self, abstract_self: JSManaged<Node>) -> DOMString {
        match self.type_id {
            ElementNodeTypeId(..) => {
                let elem: JSManaged<Element> = ElementCast::to(abstract_self);
                elem.value().TagName()
            }
            CommentNodeTypeId => ~"#comment",
            TextNodeTypeId => ~"#text",
            DoctypeNodeTypeId => {
                let doctype: JSManaged<DocumentType> = DocumentTypeCast::to(abstract_self);
                doctype.value().name.clone()
            },
            DocumentFragmentNodeTypeId => ~"#document-fragment",
            DocumentNodeTypeId(_) => ~"#document"
        }
    }

    pub fn GetBaseURI(&self) -> Option<DOMString> {
        None
    }

    pub fn GetOwnerDocument(&self) -> Option<JSManaged<Document>> {
        match self.type_id {
            ElementNodeTypeId(..) |
            CommentNodeTypeId |
            TextNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc()),
            DocumentNodeTypeId(_) => None
        }
    }

    pub fn GetParentNode(&self) -> Option<JSManaged<Node>> {
        self.parent_node
    }

    pub fn GetParentElement(&self) -> Option<JSManaged<Element>> {
        self.parent_node.filtered(|parent| parent.is_element())
                        .map(|node| ElementCast::to(node))
    }

    pub fn HasChildNodes(&self) -> bool {
        self.first_child.is_some()
    }

    pub fn GetFirstChild(&self) -> Option<JSManaged<Node>> {
        self.first_child
    }

    pub fn GetLastChild(&self) -> Option<JSManaged<Node>> {
        self.last_child
    }

    pub fn GetPreviousSibling(&self) -> Option<JSManaged<Node>> {
        self.prev_sibling
    }

    pub fn GetNextSibling(&self) -> Option<JSManaged<Node>> {
        self.next_sibling
    }

    pub fn GetNodeValue(&self, abstract_self: JSManaged<Node>) -> Option<DOMString> {
        match self.type_id {
            // ProcessingInstruction
            CommentNodeTypeId | TextNodeTypeId => {
                let chardata: JSManaged<CharacterData> = CharacterDataCast::to(abstract_self);
                Some(chardata.value().Data())
            }
            _ => {
                None
            }
        }
    }

    pub fn SetNodeValue(&mut self, _abstract_self: JSManaged<Node>, _val: Option<DOMString>)
                        -> ErrorResult {
        Ok(())
    }

    pub fn GetTextContent(&self, abstract_self: JSManaged<Node>) -> Option<DOMString> {
        match self.type_id {
          DocumentFragmentNodeTypeId | ElementNodeTypeId(..) => {
            let mut content = ~"";
            for node in abstract_self.traverse_preorder() {
                if node.is_text() {
                    let text: JSManaged<Text> = TextCast::to(node);
                    content = content + text.value().element.Data();
                }
            }
            Some(content)
          }
          CommentNodeTypeId | TextNodeTypeId => {
              let characterdata: JSManaged<CharacterData> = CharacterDataCast::to(abstract_self);
              Some(characterdata.value().Data())
          }
          DoctypeNodeTypeId | DocumentNodeTypeId(_) => {
            None
          }
        }
    }

    pub fn ChildNodes(&mut self, abstract_self: JSManaged<Node>) -> @mut NodeList {
        match self.child_list {
            None => {
                let window = self.owner_doc().value().window;
                let list = NodeList::new_child_list(window, abstract_self);
                self.child_list = Some(list);
                list
            }
            Some(list) => list
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    fn adopt(node: JSManaged<Node>, document: JSManaged<Document>) {
        // Step 1.
        match node.parent_node() {
            Some(parent) => Node::remove(node, parent, false),
            None => (),
        }

        // Step 2.
        if node.value().owner_doc() != document {
            //XXXjdm traverse_preorder balks at mutable references
            let mut nodes = ~[];
            gather_abstract_nodes(&node, &mut nodes, false);
            for descendant in nodes.mut_iter() {
                descendant.mut_value().set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node: JSManaged<Node>, parent: JSManaged<Node>, child: Option<JSManaged<Node>>)
                  -> Fallible<JSManaged<Node>> {
        fn is_inclusive_ancestor_of(node: JSManaged<Node>, parent: JSManaged<Node>) -> bool {
            node == parent || parent.ancestors().any(|ancestor| ancestor == node)
        }

        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId(..) |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => {
                return Err(HierarchyRequest);
            },
        }

        // Step 2.
        if is_inclusive_ancestor_of(node, parent) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        match child {
            Some(child) => {
                if child.parent_node() != Some(parent) {
                    return Err(NotFound);
                }
            },
            None => (),
        }

        // Step 4.
        match node.type_id() {
            DocumentFragmentNodeTypeId |
            DoctypeNodeTypeId |
            ElementNodeTypeId(_) |
            TextNodeTypeId |
            // ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId(..) => return Err(HierarchyRequest),
        }
        
        // Step 5.
        match node.type_id() {
            TextNodeTypeId => {
                match node.parent_node() {
                    Some(parent) if parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            },
            DoctypeNodeTypeId => {
                match node.parent_node() {
                    Some(parent) if !parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            },
            _ => (),
        }

        // Step 6.
        match parent.type_id() {
            DocumentNodeTypeId(_) => {
                fn inclusively_followed_by_doctype(child: Option<JSManaged<Node>>) -> bool{
                    match child {
                        Some(child) if child.is_doctype() => true,
                        Some(child) => {
                            let mut iter = child;
                            loop {
                                match iter.next_sibling() {
                                    Some(sibling) => {
                                        if sibling.is_doctype() {
                                            return true;
                                        }
                                        iter = sibling;
                                    },
                                    None => return false,
                                }
                            }
                        },
                        None => false,
                    }
                }

                match node.type_id() {
                    // Step 6.1
                    DocumentFragmentNodeTypeId => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().len() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                                // will be fixed
                                if parent.child_elements().len() > 0 {
                                    return Err(HierarchyRequest);
                                }
                                if inclusively_followed_by_doctype(child) {
                                    return Err(HierarchyRequest);
                                }
                            },
                            // Step 6.1.1(a)
                            _ => return Err(HierarchyRequest),
                        }
                    },
                    // Step 6.2
                    ElementNodeTypeId(_) => {
                        // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                        // will be fixed
                        if parent.child_elements().len() > 0 {
                            return Err(HierarchyRequest);
                        }
                        if inclusively_followed_by_doctype(child) {
                            return Err(HierarchyRequest);
                        }
                    },
                    // Step 6.3
                    DoctypeNodeTypeId => {
                        if parent.children().any(|c| c.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                        match child {
                            Some(child) => {
                                if parent.children()
                                    .take_while(|&c| c != child)
                                    .any(|c| c.is_element()) {
                                    return Err(HierarchyRequest);
                                }
                            },
                            None => {
                                // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                                // will be fixed
                                if parent.child_elements().len() > 0 {
                                    return Err(HierarchyRequest);
                                }
                            },
                        }
                    },
                    TextNodeTypeId |
                    // ProcessingInstructionNodeTypeId |
                    CommentNodeTypeId => (),
                    DocumentNodeTypeId(_) => unreachable!(),
                }
            },
            _ => (),
        }

        // Step 7-8.
        let referenceChild = if child != Some(node) {
            child
        } else {
            node.next_sibling()
        };

        // Step 9.
        Node::adopt(node, parent.value().owner_doc());

        // Step 10.
        Node::insert(node, parent, referenceChild, false);

        // Step 11.
        return Ok(node)
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: JSManaged<Node>,
              parent: JSManaged<Node>,
              child: Option<JSManaged<Node>>,
              suppress_observers: bool) {
        // XXX assert owner_doc
        // Step 1-3: ranges.
        // Step 4.
        let mut nodes = match node.type_id() {
            DocumentFragmentNodeTypeId => node.children().collect(),
            _ => ~[node],
        };

        // Step 5: DocumentFragment, mutation records.
        // Step 6: DocumentFragment.
        match node.type_id() {
            DocumentFragmentNodeTypeId => {
                for c in node.children() {
                    Node::remove(c, node, true);
                }
            },
            _ => (),
        }

        // Step 7: mutation records.
        // Step 8.
        for node in nodes.mut_iter() {
            parent.add_child(*node, child);
            node.mut_value().flags.set_is_in_doc(parent.is_in_doc());
        }

        // Step 9.
        if !suppress_observers {
            for node in nodes.iter() {
                node.node_inserted();
            }
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-replace-all
    pub fn replace_all(node: Option<JSManaged<Node>>, parent: JSManaged<Node>) {
        // Step 1.
        match node {
            Some(node) => Node::adopt(node, parent.value().owner_doc()),
            None => (),
        }

        // Step 2.
        let removedNodes: ~[JSManaged<Node>] = parent.children().collect();

        // Step 3.
        let addedNodes = match node {
            None => ~[],
            Some(node) => match node.type_id() {
                DocumentFragmentNodeTypeId => node.children().collect(),
                _ => ~[node],
            },
        };

        // Step 4.
        for child in parent.children() {
            Node::remove(child, parent, true);
        }

        // Step 5.
        match node {
            Some(node) => Node::insert(node, parent, None, true),
            None => (),
        }

        // Step 6: mutation records.

        // Step 7.
        for removedNode in removedNodes.iter() {
            removedNode.node_removed();
        }
        for addedNode in addedNodes.iter() {
            addedNode.node_inserted();
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-remove
    fn pre_remove(child: JSManaged<Node>, parent: JSManaged<Node>) -> Fallible<JSManaged<Node>> {
        // Step 1.
        if child.parent_node() != Some(parent) {
            return Err(NotFound);
        }

        // Step 2.
        Node::remove(child, parent, false);

        // Step 3.
        Ok(child)
    }

    // http://dom.spec.whatwg.org/#concept-node-remove
    fn remove(mut node: JSManaged<Node>, parent: JSManaged<Node>, suppress_observers: bool) {
        assert!(node.parent_node() == Some(parent));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node);
        node.mut_value().flags.set_is_in_doc(false);

        // Step 9.
        if !suppress_observers {
            node.node_removed();
        }
    }

    pub fn SetTextContent(&mut self, abstract_self: JSManaged<Node>, value: Option<DOMString>)
                          -> ErrorResult {
        let value = null_str_as_empty(&value);
        match self.type_id {
          DocumentFragmentNodeTypeId | ElementNodeTypeId(..) => {
            // Step 1-2.
            let node = if value.len() == 0 {
                None
            } else {
                let document = self.owner_doc();
                Some(NodeCast::from(document.value().CreateTextNode(document, value)))
            };
            // Step 3.
            Node::replace_all(node, abstract_self);
          }
          CommentNodeTypeId | TextNodeTypeId => {
            self.wait_until_safe_to_modify_dom();

            let mut characterdata: JSManaged<CharacterData> = CharacterDataCast::to(abstract_self);
            characterdata.mut_value().data = value.clone();

            // Notify the document that the content of this node is different
            let document = self.owner_doc();
            document.value().content_changed();
          }
          DoctypeNodeTypeId | DocumentNodeTypeId(_) => {}
        }
        Ok(())
    }

    pub fn InsertBefore(&self, node: JSManaged<Node>, child: Option<JSManaged<Node>>)
                        -> Fallible<JSManaged<Node>> {
        Node::pre_insert(node, node, child)
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        let document = self.owner_doc();
        document.value().wait_until_safe_to_modify_dom();
    }

    pub fn AppendChild(&self, abstract_self: JSManaged<Node>, node: JSManaged<Node>)
                       -> Fallible<JSManaged<Node>> {
        Node::pre_insert(node, abstract_self, None)
    }

    pub fn ReplaceChild(&mut self, _node: JSManaged<Node>, _child: JSManaged<Node>)
                        -> Fallible<JSManaged<Node>> {
        fail!("stub")
    }

    pub fn RemoveChild(&self, abstract_self: JSManaged<Node>, node: JSManaged<Node>)
                       -> Fallible<JSManaged<Node>> {
        Node::pre_remove(node, abstract_self)
    }

    pub fn Normalize(&mut self) {
    }

    pub fn CloneNode(&self, _deep: bool) -> Fallible<JSManaged<Node>> {
        fail!("stub")
    }

    pub fn IsEqualNode(&self, _node: Option<JSManaged<Node>>) -> bool {
        false
    }

    pub fn CompareDocumentPosition(&self, _other: JSManaged<Node>) -> u16 {
        0
    }

    pub fn Contains(&self, _other: Option<JSManaged<Node>>) -> bool {
        false
    }

    pub fn LookupPrefix(&self, _prefix: Option<DOMString>) -> Option<DOMString> {
        None
    }

    pub fn LookupNamespaceURI(&self, _namespace: Option<DOMString>) -> Option<DOMString> {
        None
    }

    pub fn IsDefaultNamespace(&self, _namespace: Option<DOMString>) -> bool {
        false
    }

    pub fn GetNamespaceURI(&self) -> Option<DOMString> {
        None
    }

    pub fn GetPrefix(&self) -> Option<DOMString> {
        None
    }

    pub fn GetLocalName(&self) -> Option<DOMString> {
        None
    }

    pub fn HasAttributes(&self) -> bool {
        false
    }


    //
    // Low-level pointer stitching
    //

    pub fn set_parent_node(&mut self, new_parent_node: Option<JSManaged<Node>>) {
        let doc = self.owner_doc();
        doc.value().wait_until_safe_to_modify_dom();
        self.parent_node = new_parent_node
    }

    pub fn set_first_child(&mut self, new_first_child: Option<JSManaged<Node>>) {
        let doc = self.owner_doc();
        doc.value().wait_until_safe_to_modify_dom();
        self.first_child = new_first_child
    }

    pub fn set_last_child(&mut self, new_last_child: Option<JSManaged<Node>>) {
        let doc = self.owner_doc();
        doc.value().wait_until_safe_to_modify_dom();
        self.last_child = new_last_child
    }

    pub fn set_prev_sibling(&mut self, new_prev_sibling: Option<JSManaged<Node>>) {
        let doc = self.owner_doc();
        doc.value().wait_until_safe_to_modify_dom();
        self.prev_sibling = new_prev_sibling
    }

    pub fn set_next_sibling(&mut self, new_next_sibling: Option<JSManaged<Node>>) {
        let doc = self.owner_doc();
        doc.value().wait_until_safe_to_modify_dom();
        self.next_sibling = new_next_sibling
    }
}

impl Reflectable for Node {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
    }
}

