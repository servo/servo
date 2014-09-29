/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::codegen::InheritTypes::{CommentCast, DocumentCast, DocumentTypeCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, NodeCast, ElementDerived};
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, NodeBase, NodeDerived};
use dom::bindings::codegen::InheritTypes::{ProcessingInstructionCast, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLLegendElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::codegen::InheritTypes::HTMLOptGroupElementDerived;
use dom::bindings::error::{Fallible, NotFound, HierarchyRequest, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, Root};
use dom::bindings::js::{OptionalSettable, TemporaryPushable, OptionalRootedRootable};
use dom::bindings::js::{ResultRootable, OptionalRootable, MutNullableJS};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::{Document, DocumentHelpers, HTMLDocument, NonHTMLDocument};
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::element::{AttributeHandlers, Element, ElementTypeId};
use dom::element::{HTMLAnchorElementTypeId, HTMLButtonElementTypeId, ElementHelpers};
use dom::element::{HTMLInputElementTypeId, HTMLSelectElementTypeId};
use dom::element::{HTMLTextAreaElementTypeId, HTMLOptGroupElementTypeId};
use dom::element::{HTMLOptionElementTypeId, HTMLFieldSetElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::nodelist::NodeList;
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use dom::window::Window;
use geom::rect::Rect;
use html::hubbub_html_parser::build_element_from_tag;
use layout_interface::{ContentBoxResponse, ContentBoxesResponse, LayoutRPC,
                       LayoutChan, ReapLayoutDataMsg, TrustedNodeAddress, UntrustedNodeAddress};
use devtools_traits::NodeInfo;
use servo_util::geometry::Au;
use servo_util::str::{DOMString, null_str_as_empty};
use style::{parse_selector_list_from_str, matches};

use js::jsapi::{JSContext, JSObject, JSTracer, JSRuntime};
use js::jsfriendapi;
use libc;
use libc::uintptr_t;
use std::cell::{RefCell, Ref, RefMut};
use std::default::Default;
use std::iter::{Map, Filter};
use std::mem;
use style;
use style::ComputedValues;
use sync::Arc;
use uuid;

//
// The basic Node structure
//

/// An HTML node.
#[jstraceable]
#[must_root]
pub struct Node {
    /// The JavaScript reflector for this node.
    pub eventtarget: EventTarget,

    /// The type of node that this is.
    type_id: NodeTypeId,

    /// The parent of this node.
    parent_node: MutNullableJS<Node>,

    /// The first child of this node.
    first_child: MutNullableJS<Node>,

    /// The last child of this node.
    last_child: MutNullableJS<Node>,

    /// The next sibling of this node.
    next_sibling: MutNullableJS<Node>,

    /// The previous sibling of this node.
    prev_sibling: MutNullableJS<Node>,

    /// The document that this node belongs to.
    owner_doc: MutNullableJS<Document>,

    /// The live list of children return by .childNodes.
    child_list: MutNullableJS<NodeList>,

    /// A bitfield of flags for node items.
    flags: RefCell<NodeFlags>,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// Must be sent back to the layout task to be destroyed when this
    /// node is finalized.
    pub layout_data: LayoutDataRef,

    unique_id: RefCell<String>,
}

impl NodeDerived for EventTarget {
    fn is_node(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(_) => true,
            _ => false
        }
    }
}

bitflags! {
    #[doc = "Flags for node items."]
    #[jstraceable]
    flags NodeFlags: u8 {
        #[doc = "Specifies whether this node is in a document."]
        static IsInDoc = 0x01,
        #[doc = "Specifies whether this node is in hover state."]
        static InHoverState = 0x02,
        #[doc = "Specifies whether this node is in disabled state."]
        static InDisabledState = 0x04,
        #[doc = "Specifies whether this node is in enabled state."]
        static InEnabledState = 0x08
    }
}

impl NodeFlags {
    pub fn new(type_id: NodeTypeId) -> NodeFlags {
        match type_id {
            DocumentNodeTypeId => IsInDoc,
            // The following elements are enabled by default.
            ElementNodeTypeId(HTMLButtonElementTypeId) |
            ElementNodeTypeId(HTMLInputElementTypeId) |
            ElementNodeTypeId(HTMLSelectElementTypeId) |
            ElementNodeTypeId(HTMLTextAreaElementTypeId) |
            ElementNodeTypeId(HTMLOptGroupElementTypeId) |
            ElementNodeTypeId(HTMLOptionElementTypeId) |
            //ElementNodeTypeId(HTMLMenuItemElementTypeId) |
            ElementNodeTypeId(HTMLFieldSetElementTypeId) => InEnabledState,
            _ => NodeFlags::empty(),
        }
    }
}

#[unsafe_destructor]
impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            self.reap_layout_data();
        }
    }
}

/// suppress observers flag
/// http://dom.spec.whatwg.org/#concept-node-insert
/// http://dom.spec.whatwg.org/#concept-node-remove
enum SuppressObserver {
    Suppressed,
    Unsuppressed
}

/// Layout data that is shared between the script and layout tasks.
pub struct SharedLayoutData {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,
}

/// Encapsulates the abstract layout data.
pub struct LayoutData {
    chan: Option<LayoutChan>,
    _shared_data: SharedLayoutData,
    _data: *const (),
}

pub struct LayoutDataRef {
    pub data_cell: RefCell<Option<LayoutData>>,
}

untraceable!(LayoutDataRef)

impl LayoutDataRef {
    pub fn new() -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(None),
        }
    }

    /// Returns true if there is layout data present.
    #[inline]
    pub fn is_present(&self) -> bool {
        self.data_cell.borrow().is_some()
    }

    /// Take the chan out of the layout data if it is present.
    pub fn take_chan(&self) -> Option<LayoutChan> {
        let mut layout_data = self.data_cell.borrow_mut();
        match &mut *layout_data {
            &None => None,
            &Some(ref mut layout_data) => Some(layout_data.chan.take().unwrap()),
        }
    }

    /// Borrows the layout data immutably, *asserting that there are no mutators*. Bad things will
    /// happen if you try to mutate the layout data while this is held. This is the only thread-
    /// safe layout data accessor.
    #[inline]
    pub unsafe fn borrow_unchecked(&self) -> *const Option<LayoutData> {
        mem::transmute(&self.data_cell)
    }

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

/// The different types of nodes.
#[deriving(PartialEq)]
#[jstraceable]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    DocumentFragmentNodeTypeId,
    CommentNodeTypeId,
    DocumentNodeTypeId,
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
    ProcessingInstructionNodeTypeId,
}

trait PrivateNodeHelpers {
    fn node_inserted(self);
    fn node_removed(self, parent_in_doc: bool);
    fn add_child(self, new_child: JSRef<Node>, before: Option<JSRef<Node>>);
    fn remove_child(self, child: JSRef<Node>);
}

impl<'a> PrivateNodeHelpers for JSRef<'a, Node> {
    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(self) {
        assert!(self.parent_node().is_some());
        let document = document_from_node(self).root();
        let is_in_doc = self.is_in_doc();

        for node in self.traverse_preorder() {
            vtable_for(&node).bind_to_tree(is_in_doc);
        }

        let parent = self.parent_node().root();
        parent.map(|parent| vtable_for(&*parent).child_inserted(self));

        document.deref().content_changed();
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(self, parent_in_doc: bool) {
        assert!(self.parent_node().is_none());
        let document = document_from_node(self).root();

        for node in self.traverse_preorder() {
            vtable_for(&node).unbind_from_tree(parent_in_doc);
        }

        document.deref().content_changed();
    }

    //
    // Pointer stitching
    //

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(self, new_child: JSRef<Node>, before: Option<JSRef<Node>>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();

        assert!(new_child.parent_node().is_none());
        assert!(new_child.prev_sibling().is_none());
        assert!(new_child.next_sibling().is_none());
        match before {
            Some(ref before) => {
                assert!(before.parent_node().root().root_ref() == Some(self));
                match before.prev_sibling().root() {
                    None => {
                        assert!(Some(*before) == self.first_child().root().root_ref());
                        self.first_child.assign(Some(new_child));
                    },
                    Some(prev_sibling) => {
                        prev_sibling.next_sibling.assign(Some(new_child));
                        new_child.prev_sibling.assign(Some(*prev_sibling));
                    },
                }
                before.prev_sibling.assign(Some(new_child));
                new_child.next_sibling.assign(Some(*before));
            },
            None => {
                match self.last_child().root() {
                    None => self.first_child.assign(Some(new_child)),
                    Some(last_child) => {
                        assert!(last_child.next_sibling().is_none());
                        last_child.next_sibling.assign(Some(new_child));
                        new_child.prev_sibling.assign(Some(*last_child));
                    }
                }

                self.last_child.assign(Some(new_child));
            },
        }

        new_child.parent_node.assign(Some(self));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node.
    fn remove_child(self, child: JSRef<Node>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();

        assert!(child.parent_node().root().root_ref() == Some(self));

        match child.prev_sibling.get().root() {
            None => {
                self.first_child.assign(child.next_sibling.get());
            }
            Some(prev_sibling) => {
                prev_sibling.next_sibling.assign(child.next_sibling.get());
            }
        }

        match child.next_sibling.get().root() {
            None => {
                self.last_child.assign(child.prev_sibling.get());
            }
            Some(next_sibling) => {
                next_sibling.prev_sibling.assign(child.prev_sibling.get());
            }
        }

        child.prev_sibling.clear();
        child.next_sibling.clear();
        child.parent_node.clear();
    }
}

pub trait NodeHelpers<'a> {
    fn ancestors(self) -> AncestorIterator<'a>;
    fn children(self) -> AbstractNodeChildrenIterator<'a>;
    fn child_elements(self) -> ChildElementIterator<'a>;
    fn following_siblings(self) -> AbstractNodeChildrenIterator<'a>;
    fn is_in_doc(self) -> bool;
    fn is_inclusive_ancestor_of(self, parent: JSRef<Node>) -> bool;
    fn is_parent_of(self, child: JSRef<Node>) -> bool;

    fn type_id(self) -> NodeTypeId;

    fn parent_node(self) -> Option<Temporary<Node>>;
    fn first_child(self) -> Option<Temporary<Node>>;
    fn last_child(self) -> Option<Temporary<Node>>;
    fn prev_sibling(self) -> Option<Temporary<Node>>;
    fn next_sibling(self) -> Option<Temporary<Node>>;

    fn owner_doc(self) -> Temporary<Document>;
    fn set_owner_doc(self, document: JSRef<Document>);
    fn is_in_html_doc(self) -> bool;

    fn wait_until_safe_to_modify_dom(self);

    fn is_element(self) -> bool;
    fn is_document(self) -> bool;
    fn is_doctype(self) -> bool;
    fn is_text(self) -> bool;
    fn is_anchor_element(self) -> bool;

    fn get_hover_state(self) -> bool;
    fn set_hover_state(self, state: bool);

    fn get_disabled_state(self) -> bool;
    fn set_disabled_state(self, state: bool);

    fn get_enabled_state(self) -> bool;
    fn set_enabled_state(self, state: bool);

    fn dump(self);
    fn dump_indent(self, indent: uint);
    fn debug_str(self) -> String;

    fn traverse_preorder(self) -> TreeIterator<'a>;
    fn sequential_traverse_postorder(self) -> TreeIterator<'a>;
    fn inclusively_following_siblings(self) -> AbstractNodeChildrenIterator<'a>;

    fn to_trusted_node_address(self) -> TrustedNodeAddress;

    fn get_bounding_content_box(self) -> Rect<Au>;
    fn get_content_boxes(self) -> Vec<Rect<Au>>;

    fn query_selector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>>;
    fn query_selector_all(self, selectors: DOMString) -> Fallible<Temporary<NodeList>>;

    fn remove_self(self);

    fn get_unique_id(self) -> String;
    fn summarize(self) -> NodeInfo;
}

impl<'a> NodeHelpers<'a> for JSRef<'a, Node> {
    /// Dumps the subtree rooted at this node, for debugging.
    fn dump(self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    fn dump_indent(self, indent: uint) {
        let mut s = String::new();
        for _ in range(0, indent) {
            s.push_str("    ");
        }

        s.push_str(self.debug_str().as_slice());
        debug!("{:s}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1u)
        }
    }

    /// Returns a string that describes this node.
    fn debug_str(self) -> String {
        format!("{:?}", self.type_id)
    }

    fn is_in_doc(self) -> bool {
        self.deref().flags.borrow().contains(IsInDoc)
    }

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(self) -> NodeTypeId {
        self.deref().type_id
    }

    fn parent_node(self) -> Option<Temporary<Node>> {
        self.deref().parent_node.get()
    }

    fn first_child(self) -> Option<Temporary<Node>> {
        self.deref().first_child.get()
    }

    fn last_child(self) -> Option<Temporary<Node>> {
        self.deref().last_child.get()
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    fn prev_sibling(self) -> Option<Temporary<Node>> {
        self.deref().prev_sibling.get()
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    fn next_sibling(self) -> Option<Temporary<Node>> {
        self.deref().next_sibling.get()
    }

    #[inline]
    fn is_element(self) -> bool {
        match self.type_id {
            ElementNodeTypeId(..) => true,
            _ => false
        }
    }

    #[inline]
    fn is_document(self) -> bool {
        self.type_id == DocumentNodeTypeId
    }

    #[inline]
    fn is_anchor_element(self) -> bool {
        self.type_id == ElementNodeTypeId(HTMLAnchorElementTypeId)
    }

    #[inline]
    fn is_doctype(self) -> bool {
        self.type_id == DoctypeNodeTypeId
    }

    #[inline]
    fn is_text(self) -> bool {
        self.type_id == TextNodeTypeId
    }

    fn get_hover_state(self) -> bool {
        self.flags.borrow().contains(InHoverState)
    }

    fn set_hover_state(self, state: bool) {
        if state {
            self.flags.borrow_mut().insert(InHoverState);
        } else {
            self.flags.borrow_mut().remove(InHoverState);
        }
    }

    fn get_disabled_state(self) -> bool {
        self.flags.borrow().contains(InDisabledState)
    }

    fn set_disabled_state(self, state: bool) {
        if state {
            self.flags.borrow_mut().insert(InDisabledState);
        } else {
            self.flags.borrow_mut().remove(InDisabledState);
        }
    }

    fn get_enabled_state(self) -> bool {
        self.flags.borrow().contains(InEnabledState)
    }

    fn set_enabled_state(self, state: bool) {
        if state {
            self.flags.borrow_mut().insert(InEnabledState);
        } else {
            self.flags.borrow_mut().remove(InEnabledState);
        }
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(self) -> TreeIterator<'a> {
        let mut nodes = vec!();
        gather_abstract_nodes(self, &mut nodes, false);
        TreeIterator::new(nodes)
    }

    /// Iterates over this node and all its descendants, in postorder.
    fn sequential_traverse_postorder(self) -> TreeIterator<'a> {
        let mut nodes = vec!();
        gather_abstract_nodes(self, &mut nodes, true);
        TreeIterator::new(nodes)
    }

    fn inclusively_following_siblings(self) -> AbstractNodeChildrenIterator<'a> {
        AbstractNodeChildrenIterator {
            current_node: Some(self.clone()),
        }
    }

    fn is_inclusive_ancestor_of(self, parent: JSRef<Node>) -> bool {
        self == parent || parent.ancestors().any(|ancestor| ancestor == self)
    }

    fn following_siblings(self) -> AbstractNodeChildrenIterator<'a> {
        AbstractNodeChildrenIterator {
            current_node: self.next_sibling().root().map(|next| next.deref().clone()),
        }
    }

    fn is_parent_of(self, child: JSRef<Node>) -> bool {
        match child.parent_node() {
            Some(ref parent) if parent == &Temporary::from_rooted(self) => true,
            _ => false
        }
    }

    fn to_trusted_node_address(self) -> TrustedNodeAddress {
        TrustedNodeAddress(self.deref() as *const Node as *const libc::c_void)
    }

    fn get_bounding_content_box(self) -> Rect<Au> {
        let window = window_from_node(self).root();
        let page = window.deref().page();
        let addr = self.to_trusted_node_address();

        let ContentBoxResponse(rect) = page.layout().content_box(addr);
        rect
    }

    fn get_content_boxes(self) -> Vec<Rect<Au>> {
        let window = window_from_node(self).root();
        let page = window.deref().page();
        let addr = self.to_trusted_node_address();
        let ContentBoxesResponse(rects) = page.layout().content_boxes(addr);
        rects
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn query_selector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        // Step 1.
        match parse_selector_list_from_str(selectors.as_slice()) {
            // Step 2.
            Err(()) => return Err(Syntax),
            // Step 3.
            Ok(ref selectors) => {
                let root = self.ancestors().last().unwrap_or(self.clone());
                for node in root.traverse_preorder() {
                    if node.is_element() && matches(selectors, &node, &mut None) {
                        let elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                        return Ok(Some(Temporary::from_rooted(elem)));
                    }
                }
            }
        }
        Ok(None)
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn query_selector_all(self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        // Step 1.
        let nodes;
        let root = self.ancestors().last().unwrap_or(self.clone());
        match parse_selector_list_from_str(selectors.as_slice()) {
            // Step 2.
            Err(()) => return Err(Syntax),
            // Step 3.
            Ok(ref selectors) => {
                nodes = root.traverse_preorder().filter(
                    // TODO(cgaebel): Is it worth it to build a bloom filter here
                    // (instead of passing `None`)? Probably.
                    |node| node.is_element() && matches(selectors, node, &mut None)).collect()
            }
        }
        let window = window_from_node(self).root();
        Ok(NodeList::new_simple_list(*window, nodes))
    }

    fn ancestors(self) -> AncestorIterator<'a> {
        AncestorIterator {
            current: self.parent_node.get().map(|node| (*node.root()).clone()),
        }
    }

    fn owner_doc(self) -> Temporary<Document> {
        self.owner_doc.get().unwrap()
    }

    fn set_owner_doc(self, document: JSRef<Document>) {
        self.owner_doc.assign(Some(document.clone()));
    }

    fn is_in_html_doc(self) -> bool {
        self.owner_doc().root().is_html_document
    }

    fn children(self) -> AbstractNodeChildrenIterator<'a> {
        AbstractNodeChildrenIterator {
            current_node: self.first_child.get().map(|node| (*node.root()).clone()),
        }
    }

    fn child_elements(self) -> ChildElementIterator<'a> {
        self.children()
            .filter(|node| {
                node.is_element()
            })
            .map(|node| {
                let elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                elem.clone()
            })
    }

    fn wait_until_safe_to_modify_dom(self) {
        let document = self.owner_doc().root();
        document.deref().wait_until_safe_to_modify_dom();
    }

    fn remove_self(self) {
        match self.parent_node().root() {
            Some(parent) => parent.remove_child(self),
            None => ()
        }
    }

    fn get_unique_id(self) -> String {
        self.unique_id.borrow().clone()
    }

    fn summarize(self) -> NodeInfo {
        if self.unique_id.borrow().is_empty() {
            let mut unique_id = self.unique_id.borrow_mut();
            *unique_id = uuid::Uuid::new_v4().to_simple_string();
        }

        NodeInfo {
            uniqueId: self.unique_id.borrow().clone(),
            baseURI: self.GetBaseURI().unwrap_or("".to_string()),
            parent: self.GetParentNode().root().map(|node| node.get_unique_id()).unwrap_or("".to_string()),
            nodeType: self.NodeType() as uint,
            namespaceURI: "".to_string(), //FIXME
            nodeName: self.NodeName(),
            numChildren: self.ChildNodes().root().Length() as uint,

            //FIXME doctype nodes only
            name: "".to_string(),
            publicId: "".to_string(),
            systemId: "".to_string(),

            attrs: if self.is_element() {
                let elem: JSRef<Element> = ElementCast::to_ref(self).unwrap();
                elem.summarize()
            } else {
                vec!()
            },

            isDocumentElement:
                self.owner_doc().root()
                    .GetDocumentElement()
                    .map(|elem| NodeCast::from_ref(*elem.root()) == self)
                    .unwrap_or(false),

            shortValue: self.GetNodeValue().unwrap_or("".to_string()), //FIXME: truncate
            incompleteValue: false, //FIXME: reflect truncation
        }
    }
}

/// If the given untrusted node address represents a valid DOM node in the given runtime,
/// returns it.
pub fn from_untrusted_node_address(runtime: *mut JSRuntime, candidate: UntrustedNodeAddress)
    -> Temporary<Node> {
    unsafe {
        let candidate: uintptr_t = mem::transmute(candidate);
        let object: *mut JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
                                                                                  candidate);
        if object.is_null() {
            fail!("Attempted to create a `JS<Node>` from an invalid pointer!")
        }
        let boxed_node: *const Node = utils::unwrap(object);
        Temporary::new(JS::from_raw(boxed_node))
    }
}

pub trait LayoutNodeHelpers {
    unsafe fn type_id_for_layout(&self) -> NodeTypeId;

    unsafe fn parent_node_ref(&self) -> Option<JS<Node>>;
    unsafe fn first_child_ref(&self) -> Option<JS<Node>>;
    unsafe fn last_child_ref(&self) -> Option<JS<Node>>;
    unsafe fn prev_sibling_ref(&self) -> Option<JS<Node>>;
    unsafe fn next_sibling_ref(&self) -> Option<JS<Node>>;

    unsafe fn owner_doc_for_layout(&self) -> JS<Document>;

    unsafe fn is_element_for_layout(&self) -> bool;
}

impl LayoutNodeHelpers for JS<Node> {
    #[inline]
    unsafe fn type_id_for_layout(&self) -> NodeTypeId {
        (*self.unsafe_get()).type_id
    }

    #[inline]
    unsafe fn is_element_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_element()
    }

    #[inline]
    unsafe fn parent_node_ref(&self) -> Option<JS<Node>> {
        (*self.unsafe_get()).parent_node.get_inner()
    }

    #[inline]
    unsafe fn first_child_ref(&self) -> Option<JS<Node>> {
        (*self.unsafe_get()).first_child.get_inner()
    }

    #[inline]
    unsafe fn last_child_ref(&self) -> Option<JS<Node>> {
        (*self.unsafe_get()).last_child.get_inner()
    }

    #[inline]
    unsafe fn prev_sibling_ref(&self) -> Option<JS<Node>> {
        (*self.unsafe_get()).prev_sibling.get_inner()
    }

    #[inline]
    unsafe fn next_sibling_ref(&self) -> Option<JS<Node>> {
        (*self.unsafe_get()).next_sibling.get_inner()
    }

    #[inline]
    unsafe fn owner_doc_for_layout(&self) -> JS<Document> {
        (*self.unsafe_get()).owner_doc.get_inner().unwrap()
    }
}

pub trait RawLayoutNodeHelpers {
    unsafe fn get_hover_state_for_layout(&self) -> bool;
    unsafe fn get_disabled_state_for_layout(&self) -> bool;
    unsafe fn get_enabled_state_for_layout(&self) -> bool;
    fn type_id_for_layout(&self) -> NodeTypeId;
}

impl RawLayoutNodeHelpers for Node {
    unsafe fn get_hover_state_for_layout(&self) -> bool {
        (*self.unsafe_get_flags()).contains(InHoverState)
    }
    unsafe fn get_disabled_state_for_layout(&self) -> bool {
        (*self.unsafe_get_flags()).contains(InDisabledState)
    }
    unsafe fn get_enabled_state_for_layout(&self) -> bool {
        (*self.unsafe_get_flags()).contains(InEnabledState)
    }

    fn type_id_for_layout(&self) -> NodeTypeId {
        self.type_id
    }
}


//
// Iteration and traversal
//

pub type ChildElementIterator<'a> = Map<'a, JSRef<'a, Node>,
                                        JSRef<'a, Element>,
                                        Filter<'a, JSRef<'a, Node>, AbstractNodeChildrenIterator<'a>>>;

pub struct AbstractNodeChildrenIterator<'a> {
    current_node: Option<JSRef<'a, Node>>,
}

impl<'a> Iterator<JSRef<'a, Node>> for AbstractNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        let node = self.current_node.clone();
        self.current_node = node.clone().and_then(|node| {
            node.next_sibling().map(|node| (*node.root()).clone())
        });
        node
    }
}

pub struct AncestorIterator<'a> {
    current: Option<JSRef<'a, Node>>,
}

impl<'a> Iterator<JSRef<'a, Node>> for AncestorIterator<'a> {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        if self.current.is_none() {
            return None;
        }

        // FIXME: Do we need two clones here?
        let x = self.current.as_ref().unwrap().clone();
        self.current = x.parent_node().map(|node| (*node.root()).clone());
        Some(x)
    }
}

// FIXME: Do this without precomputing a vector of refs.
// Easy for preorder; harder for postorder.
pub struct TreeIterator<'a> {
    nodes: Vec<JSRef<'a, Node>>,
    index: uint,
}

impl<'a> TreeIterator<'a> {
    fn new(nodes: Vec<JSRef<'a, Node>>) -> TreeIterator<'a> {
        TreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl<'a> Iterator<JSRef<'a, Node>> for TreeIterator<'a> {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let v = self.nodes[self.index];
            let v = v.clone();
            self.index += 1;
            Some(v)
        }
    }
}

#[must_root]
pub struct NodeIterator {
    pub start_node: JS<Node>,
    pub current_node: Option<JS<Node>>,
    pub depth: uint,
    include_start: bool,
    include_descendants_of_void: bool
}

impl NodeIterator {
    pub fn new<'a>(start_node: JSRef<'a, Node>,
                   include_start: bool,
                   include_descendants_of_void: bool) -> NodeIterator {
        NodeIterator {
            start_node: JS::from_rooted(start_node),
            current_node: None,
            depth: 0,
            include_start: include_start,
            include_descendants_of_void: include_descendants_of_void
        }
    }

    fn next_child<'b>(&self, node: JSRef<'b, Node>) -> Option<JSRef<'b, Node>> {
        if !self.include_descendants_of_void && node.is_element() {
            let elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
            if elem.is_void() {
                None
            } else {
                node.first_child().map(|child| (*child.root()).clone())
            }
        } else {
            node.first_child().map(|child| (*child.root()).clone())
        }
    }
}

impl<'a> Iterator<JSRef<'a, Node>> for NodeIterator {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        self.current_node = match self.current_node.as_ref().map(|node| node.root()) {
            None => {
                if self.include_start {
                    Some(self.start_node)
                } else {
                    self.next_child(*self.start_node.root())
                        .map(|child| JS::from_rooted(child))
                }
            },
            Some(node) => {
                match self.next_child(*node) {
                    Some(child) => {
                        self.depth += 1;
                        Some(JS::from_rooted(child))
                    },
                    None if JS::from_rooted(*node) == self.start_node => None,
                    None => {
                        match node.deref().next_sibling().root() {
                            Some(sibling) => Some(JS::from_rooted(*sibling)),
                            None => {
                                let mut candidate = node.deref().clone();
                                while candidate.next_sibling().is_none() {
                                    candidate = (*candidate.parent_node()
                                                          .expect("Got to root without reaching start node")
                                                          .root()).clone();
                                    self.depth -= 1;
                                    if JS::from_rooted(candidate) == self.start_node {
                                        break;
                                    }
                                }
                                if JS::from_rooted(candidate) != self.start_node {
                                    candidate.next_sibling().map(|node| JS::from_rooted(*node.root().deref()))
                                } else {
                                    None
                                }
                            }
                        }
                    }
                }
            }
        };
        self.current_node.map(|node| (*node.root()).clone())
    }
}

fn gather_abstract_nodes<'a>(cur: JSRef<'a, Node>, refs: &mut Vec<JSRef<'a, Node>>, postorder: bool) {
    if !postorder {
        refs.push(cur.clone());
    }
    for kid in cur.children() {
        gather_abstract_nodes(kid, refs, postorder)
    }
    if postorder {
        refs.push(cur.clone());
    }
}

/// Specifies whether children must be recursively cloned or not.
#[deriving(PartialEq)]
pub enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren
}

fn as_uintptr<T>(t: &T) -> uintptr_t { t as *const T as uintptr_t }

impl Node {
    pub fn reflect_node<N: Reflectable+NodeBase>
            (node:      Box<N>,
             document:  JSRef<Document>,
             wrap_fn:   extern "Rust" fn(*mut JSContext, &GlobalRef, Box<N>) -> Temporary<N>)
             -> Temporary<N> {
        let window = document.window.root();
        reflect_dom_object(node, &global::Window(*window), wrap_fn)
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: JSRef<Document>) -> Node {
        Node::new_(type_id, Some(doc.clone()))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<JSRef<Document>>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(NodeTargetTypeId(type_id)),
            type_id: type_id,

            parent_node: Default::default(),
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            owner_doc: MutNullableJS::new(doc),
            child_list: Default::default(),

            flags: RefCell::new(NodeFlags::new(type_id)),

            layout_data: LayoutDataRef::new(),

            unique_id: RefCell::new("".to_string()),
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node: JSRef<Node>, document: JSRef<Document>) {
        // Step 1.
        match node.parent_node().root() {
            Some(parent) => {
                Node::remove(node, *parent, Unsuppressed);
            }
            None => (),
        }

        // Step 2.
        let node_doc = document_from_node(node).root();
        if *node_doc != document {
            for descendant in node.traverse_preorder() {
                descendant.set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node: JSRef<Node>, parent: JSRef<Node>, child: Option<JSRef<Node>>)
                  -> Fallible<Temporary<Node>> {
        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(parent) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        match child {
            Some(child) if !parent.is_parent_of(child) => return Err(NotFound),
            _ => ()
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId => {
                match node.parent_node().root() {
                    Some(ref parent) if parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DoctypeNodeTypeId => {
                match node.parent_node().root() {
                    Some(ref parent) if !parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(_) |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId => return Err(HierarchyRequest)
        }

        // Step 6.
        match parent.type_id() {
            DocumentNodeTypeId => {
                match node.type_id() {
                    // Step 6.1
                    DocumentFragmentNodeTypeId => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().count() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                                // will be fixed
                                if parent.child_elements().count() > 0 {
                                    return Err(HierarchyRequest);
                                }
                                match child {
                                    Some(child) => {
                                        if child.inclusively_following_siblings()
                                            .any(|child| child.is_doctype()) {
                                                return Err(HierarchyRequest)
                                            }
                                    }
                                    _ => (),
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
                        if parent.child_elements().count() > 0 {
                            return Err(HierarchyRequest);
                        }
                        match child {
                            Some(ref child) => {
                                if child.inclusively_following_siblings()
                                    .any(|child| child.is_doctype()) {
                                        return Err(HierarchyRequest)
                                    }
                            }
                            _ => (),
                        }
                    },
                    // Step 6.3
                    DoctypeNodeTypeId => {
                        if parent.children().any(|c| c.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                        match child {
                            Some(ref child) => {
                                if parent.children()
                                    .take_while(|c| c != child)
                                    .any(|c| c.is_element()) {
                                    return Err(HierarchyRequest);
                                }
                            },
                            None => {
                                // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                                // will be fixed
                                if parent.child_elements().count() > 0 {
                                    return Err(HierarchyRequest);
                                }
                            },
                        }
                    },
                    TextNodeTypeId |
                    ProcessingInstructionNodeTypeId |
                    CommentNodeTypeId => (),
                    DocumentNodeTypeId => unreachable!(),
                }
            },
            _ => (),
        }

        // Step 7-8.
        let referenceChild = match child {
            Some(child) if child == node => node.next_sibling().map(|node| (*node.root()).clone()),
            _ => child
        };

        // Step 9.
        let document = document_from_node(parent).root();
        Node::adopt(node, *document);

        // Step 10.
        Node::insert(node, parent, referenceChild, Unsuppressed);

        // Step 11.
        return Ok(Temporary::from_rooted(node))
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: JSRef<Node>,
              parent: JSRef<Node>,
              child: Option<JSRef<Node>>,
              suppress_observers: SuppressObserver) {
        // XXX assert owner_doc
        // Step 1-3: ranges.
        // Step 4.
        let mut nodes = match node.type_id() {
            DocumentFragmentNodeTypeId => node.children().collect(),
            _ => vec!(node.clone()),
        };

        // Step 5: DocumentFragment, mutation records.
        // Step 6: DocumentFragment.
        match node.type_id() {
            DocumentFragmentNodeTypeId => {
                for c in node.children() {
                    Node::remove(c, node, Suppressed);
                }
            },
            _ => (),
        }

        // Step 7: mutation records.
        // Step 8.
        for node in nodes.iter_mut() {
            parent.add_child(*node, child);
            let is_in_doc = parent.is_in_doc();
            for kid in node.traverse_preorder() {
                if is_in_doc {
                    kid.flags.borrow_mut().insert(IsInDoc);
                } else {
                    kid.flags.borrow_mut().remove(IsInDoc);
                }
            }
        }

        // Step 9.
        match suppress_observers {
            Unsuppressed => {
                for node in nodes.iter() {
                    node.node_inserted();
                }
            }
            Suppressed => ()
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-replace-all
    fn replace_all(node: Option<JSRef<Node>>, parent: JSRef<Node>) {

        // Step 1.
        match node {
            Some(node) => {
                let document = document_from_node(parent).root();
                Node::adopt(node, *document);
            }
            None => (),
        }

        // Step 2.
        let removedNodes: Vec<JSRef<Node>> = parent.children().collect();

        // Step 3.
        let addedNodes = match node {
            None => vec!(),
            Some(node) => match node.type_id() {
                DocumentFragmentNodeTypeId => node.children().collect(),
                _ => vec!(node.clone()),
            },
        };

        // Step 4.
        for child in parent.children() {
            Node::remove(child, parent, Suppressed);
        }

        // Step 5.
        match node {
            Some(node) => Node::insert(node, parent, None, Suppressed),
            None => (),
        }

        // Step 6: mutation records.

        // Step 7.
        let parent_in_doc = parent.is_in_doc();
        for removedNode in removedNodes.iter() {
            removedNode.node_removed(parent_in_doc);
        }
        for addedNode in addedNodes.iter() {
            addedNode.node_inserted();
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-remove
    fn pre_remove(child: JSRef<Node>, parent: JSRef<Node>) -> Fallible<Temporary<Node>> {
        // Step 1.
        match child.parent_node() {
            Some(ref node) if node != &Temporary::from_rooted(parent) => return Err(NotFound),
            _ => ()
        }

        // Step 2.
        Node::remove(child, parent, Unsuppressed);

        // Step 3.
        Ok(Temporary::from_rooted(child))
    }

    // http://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node: JSRef<Node>, parent: JSRef<Node>, suppress_observers: SuppressObserver) {
        assert!(node.parent_node().map_or(false, |node_parent| node_parent == Temporary::from_rooted(parent)));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node);

        node.deref().flags.borrow_mut().remove(IsInDoc);

        // Step 9.
        match suppress_observers {
            Suppressed => (),
            Unsuppressed => node.node_removed(parent.is_in_doc()),
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-clone
    pub fn clone(node: JSRef<Node>, maybe_doc: Option<JSRef<Document>>,
                 clone_children: CloneChildrenFlag) -> Temporary<Node> {

        // Step 1.
        let document = match maybe_doc {
            Some(doc) => JS::from_rooted(doc).root(),
            None => node.owner_doc().root()
        };

        // Step 2.
        // XXXabinader: clone() for each node as trait?
        let copy: Root<Node> = match node.type_id() {
            DoctypeNodeTypeId => {
                let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
                let doctype = doctype.deref();
                let doctype = DocumentType::new(doctype.name.clone(),
                                                Some(doctype.public_id.clone()),
                                                Some(doctype.system_id.clone()), *document);
                NodeCast::from_temporary(doctype)
            },
            DocumentFragmentNodeTypeId => {
                let doc_fragment = DocumentFragment::new(*document);
                NodeCast::from_temporary(doc_fragment)
            },
            CommentNodeTypeId => {
                let comment: JSRef<Comment> = CommentCast::to_ref(node).unwrap();
                let comment = comment.deref();
                let comment = Comment::new(comment.characterdata.data.borrow().clone(), *document);
                NodeCast::from_temporary(comment)
            },
            DocumentNodeTypeId => {
                let document: JSRef<Document> = DocumentCast::to_ref(node).unwrap();
                let is_html_doc = match document.is_html_document {
                    true => HTMLDocument,
                    false => NonHTMLDocument
                };
                let window = document.window.root();
                let document = Document::new(*window, Some(document.url().clone()),
                                             is_html_doc, None);
                NodeCast::from_temporary(document)
            },
            ElementNodeTypeId(..) => {
                let element: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let element = element.deref();
                let element = build_element_from_tag(element.local_name.as_slice().to_string(),
                    element.namespace.clone(), *document);
                NodeCast::from_temporary(element)
            },
            TextNodeTypeId => {
                let text: JSRef<Text> = TextCast::to_ref(node).unwrap();
                let text = text.deref();
                let text = Text::new(text.characterdata.data.borrow().clone(), *document);
                NodeCast::from_temporary(text)
            },
            ProcessingInstructionNodeTypeId => {
                let pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
                let pi = pi.deref();
                let pi = ProcessingInstruction::new(pi.target.clone(),
                                                    pi.characterdata.data.borrow().clone(), *document);
                NodeCast::from_temporary(pi)
            },
        }.root();

        // Step 3.
        let document = if copy.is_document() {
            let doc: JSRef<Document> = DocumentCast::to_ref(*copy).unwrap();
            JS::from_rooted(doc).root()
        } else {
            JS::from_rooted(*document).root()
        };
        assert!(&*copy.owner_doc().root() == &*document);

        // Step 4 (some data already copied in step 2).
        match node.type_id() {
            DocumentNodeTypeId => {
                let node_doc: JSRef<Document> = DocumentCast::to_ref(node).unwrap();
                let copy_doc: JSRef<Document> = DocumentCast::to_ref(*copy).unwrap();
                copy_doc.set_encoding_name(node_doc.encoding_name.borrow().clone());
                copy_doc.set_quirks_mode(node_doc.quirks_mode());
            },
            ElementNodeTypeId(..) => {
                let node_elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let copy_elem: JSRef<Element> = ElementCast::to_ref(*copy).unwrap();

                // FIXME: https://github.com/mozilla/servo/issues/1737
                let window = document.deref().window.root();
                for attr in node_elem.deref().attrs.borrow().iter().map(|attr| attr.root()) {
                    copy_elem.deref().attrs.borrow_mut().push_unrooted(
                        &Attr::new(*window,
                                   attr.local_name().clone(), attr.deref().value().clone(),
                                   attr.deref().name.clone(), attr.deref().namespace.clone(),
                                   attr.deref().prefix.clone(), copy_elem));
                }
            },
            _ => ()
        }

        // Step 5: cloning steps.

        // Step 6.
        if clone_children == CloneChildren {
            for child in node.children() {
                let child_copy = Node::clone(child, Some(*document), clone_children).root();
                let _inserted_node = Node::pre_insert(*child_copy, *copy, None);
            }
        }

        // Step 7.
        Temporary::from_rooted(*copy)
    }

    /// Sends layout data, if any, back to the layout task to be destroyed.
    unsafe fn reap_layout_data(&mut self) {
        if self.layout_data.is_present() {
            let layout_data = mem::replace(&mut self.layout_data, LayoutDataRef::new());
            let layout_chan = layout_data.take_chan();
            match layout_chan {
                None => {}
                Some(chan) => {
                    let LayoutChan(chan) = chan;
                    chan.send(ReapLayoutDataMsg(layout_data))
                },
            }
        }
    }

    pub unsafe fn unsafe_get_flags(&self) -> *const NodeFlags {
        mem::transmute(&self.flags)
    }

    pub fn collect_text_contents<'a, T: Iterator<JSRef<'a, Node>>>(mut iterator: T) -> String {
        let mut content = String::new();
        for node in iterator {
            let text: Option<JSRef<Text>> = TextCast::to_ref(node);
            match text {
                Some(text) => content.push_str(text.characterdata.data.borrow().as_slice()),
                None => (),
            }
        }
        content
    }
}

impl<'a> NodeMethods for JSRef<'a, Node> {
    // http://dom.spec.whatwg.org/#dom-node-nodetype
    fn NodeType(self) -> u16 {
        match self.type_id {
            ElementNodeTypeId(_)            => NodeConstants::ELEMENT_NODE,
            TextNodeTypeId                  => NodeConstants::TEXT_NODE,
            ProcessingInstructionNodeTypeId => NodeConstants::PROCESSING_INSTRUCTION_NODE,
            CommentNodeTypeId               => NodeConstants::COMMENT_NODE,
            DocumentNodeTypeId              => NodeConstants::DOCUMENT_NODE,
            DoctypeNodeTypeId               => NodeConstants::DOCUMENT_TYPE_NODE,
            DocumentFragmentNodeTypeId      => NodeConstants::DOCUMENT_FRAGMENT_NODE,
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodename
    fn NodeName(self) -> DOMString {
        match self.type_id {
            ElementNodeTypeId(..) => {
                let elem: JSRef<Element> = ElementCast::to_ref(self).unwrap();
                elem.TagName()
            }
            TextNodeTypeId => "#text".to_string(),
            ProcessingInstructionNodeTypeId => {
                let processing_instruction: JSRef<ProcessingInstruction> =
                    ProcessingInstructionCast::to_ref(self).unwrap();
                processing_instruction.Target()
            }
            CommentNodeTypeId => "#comment".to_string(),
            DoctypeNodeTypeId => {
                let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(self).unwrap();
                doctype.deref().name.clone()
            },
            DocumentFragmentNodeTypeId => "#document-fragment".to_string(),
            DocumentNodeTypeId => "#document".to_string()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-baseuri
    fn GetBaseURI(self) -> Option<DOMString> {
        // FIXME (#1824) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-ownerdocument
    fn GetOwnerDocument(self) -> Option<Temporary<Document>> {
        match self.type_id {
            ElementNodeTypeId(..) |
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc()),
            DocumentNodeTypeId => None
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-parentnode
    fn GetParentNode(self) -> Option<Temporary<Node>> {
        self.parent_node.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-parentelement
    fn GetParentElement(self) -> Option<Temporary<Element>> {
        self.parent_node.get()
                        .and_then(|parent| {
                            let parent = parent.root();
                            ElementCast::to_ref(*parent).map(|elem| {
                                Temporary::from_rooted(elem)
                            })
                        })
    }

    // http://dom.spec.whatwg.org/#dom-node-haschildnodes
    fn HasChildNodes(self) -> bool {
        self.first_child.get().is_some()
    }

    // http://dom.spec.whatwg.org/#dom-node-childnodes
    fn ChildNodes(self) -> Temporary<NodeList> {
        match self.child_list.get() {
            None => (),
            Some(list) => return list,
        }

        let doc = self.owner_doc().root();
        let window = doc.deref().window.root();
        let child_list = NodeList::new_child_list(*window, self);
        self.child_list.assign(Some(child_list));
        self.child_list.get().unwrap()
    }

    // http://dom.spec.whatwg.org/#dom-node-firstchild
    fn GetFirstChild(self) -> Option<Temporary<Node>> {
        self.first_child.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-lastchild
    fn GetLastChild(self) -> Option<Temporary<Node>> {
        self.last_child.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-previoussibling
    fn GetPreviousSibling(self) -> Option<Temporary<Node>> {
        self.prev_sibling.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-nextsibling
    fn GetNextSibling(self) -> Option<Temporary<Node>> {
        self.next_sibling.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    fn GetNodeValue(self) -> Option<DOMString> {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                let chardata: JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                Some(chardata.Data())
            }
            _ => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    fn SetNodeValue(self, val: Option<DOMString>) {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.SetTextContent(val)
            }
            _ => {}
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    fn GetTextContent(self) -> Option<DOMString> {
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                let content = Node::collect_text_contents(self.traverse_preorder());
                Some(content)
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                Some(characterdata.Data())
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    fn SetTextContent(self, value: Option<DOMString>) {
        let value = null_str_as_empty(&value);
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                // Step 1-2.
                let node = if value.len() == 0 {
                    None
                } else {
                    let document = self.owner_doc().root();
                    Some(NodeCast::from_temporary(document.deref().CreateTextNode(value)))
                }.root();

                // Step 3.
                Node::replace_all(node.root_ref(), self);
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.wait_until_safe_to_modify_dom();

                let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                *characterdata.data.borrow_mut() = value;

                // Notify the document that the content of this node is different
                let document = self.owner_doc().root();
                document.deref().content_changed();
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId => {}
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-insertbefore
    fn InsertBefore(self, node: JSRef<Node>, child: Option<JSRef<Node>>) -> Fallible<Temporary<Node>> {
        Node::pre_insert(node, self, child)
    }

    // http://dom.spec.whatwg.org/#dom-node-appendchild
    fn AppendChild(self, node: JSRef<Node>) -> Fallible<Temporary<Node>> {
        Node::pre_insert(node, self, None)
    }

    // http://dom.spec.whatwg.org/#concept-node-replace
    fn ReplaceChild(self, node: JSRef<Node>, child: JSRef<Node>) -> Fallible<Temporary<Node>> {

        // Step 1.
        match self.type_id {
            DocumentNodeTypeId |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(self) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        if !self.is_parent_of(child) {
            return Err(NotFound);
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId if self.is_document() => return Err(HierarchyRequest),
            DoctypeNodeTypeId if !self.is_document() => return Err(HierarchyRequest),
            DocumentFragmentNodeTypeId |
            DoctypeNodeTypeId |
            ElementNodeTypeId(..) |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId => return Err(HierarchyRequest)
        }

        // Step 6.
        match self.type_id {
            DocumentNodeTypeId => {
                match node.type_id() {
                    // Step 6.1
                    DocumentFragmentNodeTypeId => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().count() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                if self.child_elements().any(|c| NodeCast::from_ref(c) != child) {
                                    return Err(HierarchyRequest);
                                }
                                if child.following_siblings()
                                        .any(|child| child.is_doctype()) {
                                    return Err(HierarchyRequest);
                                }
                            },
                            // Step 6.1.1(a)
                            _ => return Err(HierarchyRequest)
                        }
                    },
                    // Step 6.2
                    ElementNodeTypeId(..) => {
                        if self.child_elements().any(|c| NodeCast::from_ref(c) != child) {
                            return Err(HierarchyRequest);
                        }
                        if child.following_siblings()
                                .any(|child| child.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    // Step 6.3
                    DoctypeNodeTypeId => {
                        if self.children().any(|c| c.is_doctype() && c != child) {
                            return Err(HierarchyRequest);
                        }
                        if self.children()
                            .take_while(|c| *c != child)
                            .any(|c| c.is_element()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    TextNodeTypeId |
                    ProcessingInstructionNodeTypeId |
                    CommentNodeTypeId => (),
                    DocumentNodeTypeId => unreachable!()
                }
            },
            _ => ()
        }

        // Ok if not caught by previous error checks.
        if node == child {
            return Ok(Temporary::from_rooted(child));
        }

        // Step 7-8.
        let next_sibling = child.next_sibling().map(|node| (*node.root()).clone());
        let reference_child = match next_sibling {
            Some(sibling) if sibling == node => node.next_sibling().map(|node| (*node.root()).clone()),
            _ => next_sibling
        };

        // Step 9.
        let document = document_from_node(self).root();
        Node::adopt(node, *document);

        {
            // Step 10.
            Node::remove(child, self, Suppressed);

            // Step 11.
            Node::insert(node, self, reference_child, Suppressed);
        }

        // Step 12-14.
        // Step 13: mutation records.
        child.node_removed(self.is_in_doc());
        if node.type_id() == DocumentFragmentNodeTypeId {
            for child_node in node.children() {
                child_node.node_inserted();
            }
        } else {
            node.node_inserted();
        }

        // Step 15.
        Ok(Temporary::from_rooted(child))
    }

    // http://dom.spec.whatwg.org/#dom-node-removechild
    fn RemoveChild(self, node: JSRef<Node>)
                       -> Fallible<Temporary<Node>> {
        Node::pre_remove(node, self)
    }

    // http://dom.spec.whatwg.org/#dom-node-normalize
    fn Normalize(self) {
        let mut prev_text = None;
        for child in self.children() {
            if child.is_text() {
                let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(child).unwrap();
                if characterdata.Length() == 0 {
                    self.remove_child(child);
                } else {
                    match prev_text {
                        Some(text_node) => {
                            let prev_characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(text_node).unwrap();
                            let _ = prev_characterdata.AppendData(characterdata.Data());
                            self.remove_child(child);
                        },
                        None => prev_text = Some(child)
                    }
                }
            } else {
                child.Normalize();
                prev_text = None;
            }

        }
    }

    // http://dom.spec.whatwg.org/#dom-node-clonenode
    fn CloneNode(self, deep: bool) -> Temporary<Node> {
        match deep {
            true => Node::clone(self, None, CloneChildren),
            false => Node::clone(self, None, DoNotCloneChildren)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-isequalnode
    fn IsEqualNode(self, maybe_node: Option<JSRef<Node>>) -> bool {
        fn is_equal_doctype(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
            let other_doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(other).unwrap();
            (doctype.deref().name == other_doctype.deref().name) &&
            (doctype.deref().public_id == other_doctype.deref().public_id) &&
            (doctype.deref().system_id == other_doctype.deref().system_id)
        }
        fn is_equal_element(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let element: JSRef<Element> = ElementCast::to_ref(node).unwrap();
            let other_element: JSRef<Element> = ElementCast::to_ref(other).unwrap();
            // FIXME: namespace prefix
            let element = element.deref();
            let other_element = other_element.deref();
            (element.namespace == other_element.namespace) &&
            (element.local_name == other_element.local_name) &&
            (element.attrs.borrow().len() == other_element.attrs.borrow().len())
        }
        fn is_equal_processinginstruction(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
            let other_pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(other).unwrap();
            (pi.deref().target == other_pi.deref().target) &&
            (*pi.deref().characterdata.data.borrow() == *other_pi.deref().characterdata.data.borrow())
        }
        fn is_equal_characterdata(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(node).unwrap();
            let other_characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(other).unwrap();
            *characterdata.deref().data.borrow() == *other_characterdata.deref().data.borrow()
        }
        fn is_equal_element_attrs(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let element: JSRef<Element> = ElementCast::to_ref(node).unwrap();
            let other_element: JSRef<Element> = ElementCast::to_ref(other).unwrap();
            let element = element.deref();
            let other_element = other_element.deref();
            assert!(element.attrs.borrow().len() == other_element.attrs.borrow().len());
            element.attrs.borrow().iter().map(|attr| attr.root()).all(|attr| {
                other_element.attrs.borrow().iter().map(|attr| attr.root()).any(|other_attr| {
                    (attr.namespace == other_attr.namespace) &&
                    (attr.local_name() == other_attr.local_name()) &&
                    (attr.deref().value().as_slice() == other_attr.deref().value().as_slice())
                })
            })
        }
        fn is_equal_node(this: JSRef<Node>, node: JSRef<Node>) -> bool {
            // Step 2.
            if this.type_id() != node.type_id() {
                return false;
            }

            match node.type_id() {
                // Step 3.
                DoctypeNodeTypeId if !is_equal_doctype(this, node) => return false,
                ElementNodeTypeId(..) if !is_equal_element(this, node) => return false,
                ProcessingInstructionNodeTypeId if !is_equal_processinginstruction(this, node) => return false,
                TextNodeTypeId |
                CommentNodeTypeId if !is_equal_characterdata(this, node) => return false,
                // Step 4.
                ElementNodeTypeId(..) if !is_equal_element_attrs(this, node) => return false,
                _ => ()
            }

            // Step 5.
            if this.children().count() != node.children().count() {
                return false;
            }

            // Step 6.
            this.children().zip(node.children()).all(|(ref child, ref other_child)| {
                is_equal_node(*child, *other_child)
            })
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(node) => is_equal_node(self, node)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    fn CompareDocumentPosition(self, other: JSRef<Node>) -> u16 {
        if self == other {
            // step 2.
            0
        } else {
            let mut lastself = self.clone();
            let mut lastother = other.clone();
            for ancestor in self.ancestors() {
                if ancestor == other {
                    // step 4.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINS +
                           NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                lastself = ancestor.clone();
            }
            for ancestor in other.ancestors() {
                if ancestor == self {
                    // step 5.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINED_BY +
                           NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
                lastother = ancestor.clone();
            }

            if lastself != lastother {
                let abstract_uint: uintptr_t = as_uintptr(&self);
                let other_uint: uintptr_t = as_uintptr(&*other);

                let random = if abstract_uint < other_uint {
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING
                } else {
                    NodeConstants::DOCUMENT_POSITION_PRECEDING
                };
                // step 3.
                return random +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC;
            }

            for child in lastself.traverse_preorder() {
                if child == other {
                    // step 6.
                    return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                if child == self {
                    // step 7.
                    return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
            }
            unreachable!()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-contains
    fn Contains(self, maybe_other: Option<JSRef<Node>>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupprefix
    fn LookupPrefix(self, _prefix: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    fn LookupNamespaceURI(self, _namespace: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    fn IsDefaultNamespace(self, _namespace: Option<DOMString>) -> bool {
        // FIXME (#1826) implement.
        false
    }
}


impl Reflectable for Node {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

pub fn document_from_node<T: NodeBase+Reflectable>(derived: JSRef<T>) -> Temporary<Document> {
    let node: JSRef<Node> = NodeCast::from_ref(derived);
    node.owner_doc()
}

pub fn window_from_node<T: NodeBase+Reflectable>(derived: JSRef<T>) -> Temporary<Window> {
    let document = document_from_node(derived).root();
    Temporary::new(document.deref().window.clone())
}

impl<'a> VirtualMethods for JSRef<'a, Node> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_borrowed_ref(self);
        Some(eventtarget as &VirtualMethods)
    }
}

impl<'a> style::TNode<'a, JSRef<'a, Element>> for JSRef<'a, Node> {
    fn parent_node(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn parent_node<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.parent_node()
        }

        parent_node(self).map(|node| *node.root())
    }

    fn first_child(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn first_child<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.first_child()
        }

        first_child(self).map(|node| *node.root())
    }

    fn prev_sibling(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn prev_sibling<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.prev_sibling()
        }

        prev_sibling(self).map(|node| *node.root())
    }

    fn next_sibling(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn next_sibling<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.next_sibling()
        }

        next_sibling(self).map(|node| *node.root())
    }

    fn is_document(self) -> bool {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn is_document<'a, T: NodeHelpers<'a>>(this: T) -> bool {
            this.is_document()
        }

        is_document(self)
    }

    fn is_element(self) -> bool {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn is_element<'a, T: NodeHelpers<'a>>(this: T) -> bool {
            this.is_element()
        }

        is_element(self)
    }

    fn as_element(self) -> JSRef<'a, Element> {
        let elem: Option<JSRef<'a, Element>> = ElementCast::to_ref(self);
        assert!(elem.is_some());
        elem.unwrap()
    }

    fn match_attr(self, attr: &style::AttrSelector, test: |&str| -> bool) -> bool {
        let name = {
            if self.is_html_element_in_html_document() {
                attr.lower_name.as_slice()
            } else {
                attr.name.as_slice()
            }
        };
        match attr.namespace {
            style::SpecificNamespace(ref ns) => {
                self.as_element().get_attribute(ns.clone(), name).root()
                    .map_or(false, |attr| test(attr.deref().Value().as_slice()))
            },
            // FIXME: https://github.com/mozilla/servo/issues/1558
            style::AnyNamespace => false,
        }
    }

    fn is_html_element_in_html_document(self) -> bool {
        let elem: Option<JSRef<'a, Element>> = ElementCast::to_ref(self);
        assert!(elem.is_some());
        elem.unwrap().html_element_in_html_document()
    }
}

pub trait DisabledStateHelpers {
    fn check_ancestors_disabled_state_for_form_control(self);
    fn check_parent_disabled_state_for_option(self);
    fn check_disabled_attribute(self);
}

impl<'a> DisabledStateHelpers for JSRef<'a, Node> {
    fn check_ancestors_disabled_state_for_form_control(self) {
        if self.get_disabled_state() { return; }
        for ancestor in self.ancestors().filter(|ancestor| ancestor.is_htmlfieldsetelement()) {
            if !ancestor.get_disabled_state() { continue; }
            if ancestor.is_parent_of(self) {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
                return;
            }
            match ancestor.children().find(|child| child.is_htmllegendelement()) {
                Some(legend) => {
                    // XXXabinader: should we save previous ancestor to avoid this iteration?
                    if self.ancestors().any(|ancestor| ancestor == legend) { continue; }
                },
                None => ()
            }
            self.set_disabled_state(true);
            self.set_enabled_state(false);
            return;
        }
    }

    fn check_parent_disabled_state_for_option(self) {
        if self.get_disabled_state() { return; }
        match self.parent_node().root() {
            Some(ref parent) if parent.is_htmloptgroupelement() && parent.get_disabled_state() => {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
            },
            _ => ()
        }
    }

    fn check_disabled_attribute(self) {
        let elem: JSRef<'a, Element> = ElementCast::to_ref(self).unwrap();
        let has_disabled_attrib = elem.has_attribute("disabled");
        self.set_disabled_state(has_disabled_attrib);
        self.set_enabled_state(!has_disabled_attrib);
    }
}
