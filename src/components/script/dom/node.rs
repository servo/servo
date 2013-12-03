/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::{DOMString, null_str_as_empty};
use dom::bindings::utils::{ErrorResult, Fallible, NotFound, HierarchyRequest};
use dom::bindings::utils;
use dom::characterdata::CharacterData;
use dom::document::{AbstractDocument, DocumentTypeId};
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementTypeId, HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::element::{HTMLAnchorElementTypeId, HTMLStyleElementTypeId};
use dom::eventtarget::{AbstractEventTarget, EventTarget, NodeTypeId};
use dom::nodelist::{NodeList};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::text::Text;

use js::jsapi::{JSObject, JSContext};
use servo_util::slot::{MutSlotRef, Slot, SlotRef};
use servo_util::tree::{TreeNode, TreeNodeRef, TreeNodeRefAsElement};
use std::cast::transmute;
use std::cast;
use std::unstable::raw::Box;
use std::util;

//
// The basic Node structure
//

/// A phantom type representing the script task's view of this node. Script is able to mutate
/// nodes but may not access layout data.
#[deriving(Eq)]
pub struct ScriptView;

/// A phantom type representing the layout task's view of the node. Layout is not allowed to mutate
/// nodes but may access layout data.
#[deriving(Eq)]
pub struct LayoutView;

// We shouldn't need Eq for ScriptView and LayoutView; see Rust #7671.

/// This is what a Node looks like if you do not know what kind of node it is. To unpack it, use
/// downcast().
///
/// FIXME: This should be replaced with a trait once they can inherit from structs.
#[deriving(Eq)]
pub struct AbstractNode<View> {
    priv obj: *mut Box<Node<View>>,
}

pub struct AbstractNodeChildrenIterator<View> {
    priv current_node: Option<AbstractNode<View>>,
}

/// An HTML node.
///
/// `View` describes extra data associated with this node that this task has access to. For
/// the script task, this is the unit type `()`. For the layout task, this is
/// `LayoutData`.
pub struct Node<View> {
    /// The JavaScript reflector for this node.
    eventtarget: EventTarget,

    /// The type of node that this is.
    type_id: NodeTypeId,

    abstract: Option<AbstractNode<View>>,

    /// The parent of this node.
    parent_node: Option<AbstractNode<View>>,

    /// The first child of this node.
    first_child: Option<AbstractNode<View>>,

    /// The last child of this node.
    last_child: Option<AbstractNode<View>>,

    /// The next sibling of this node.
    next_sibling: Option<AbstractNode<View>>,

    /// The previous sibling of this node.
    prev_sibling: Option<AbstractNode<View>>,

    /// The document that this node belongs to.
    priv owner_doc: Option<AbstractDocument>,

    /// The live list of children return by .childNodes.
    child_list: Option<@mut NodeList>,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// FIXME(pcwalton): We need to send these back to the layout task to be destroyed when this
    /// node is finalized.
    layout_data: LayoutDataRef,
}

#[unsafe_destructor]
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        unsafe {
            let this: &mut Node<ScriptView> = cast::transmute(self);
            this.reap_layout_data()
        }
    }
}

/// Encapsulates the abstract layout data.
pub struct LayoutDataRef {
    priv data: Slot<Option<*()>>,
}

impl LayoutDataRef {
    #[inline]
    pub fn init() -> LayoutDataRef {
        LayoutDataRef {
            data: Slot::init(None),
        }
    }

    /// Creates a new piece of layout data from a value.
    #[inline]
    pub unsafe fn from_data<T>(data: ~T) -> LayoutDataRef {
        LayoutDataRef {
            data: Slot::init(Some(cast::transmute(data))),
        }
    }

    /// Returns true if this layout data is present or false otherwise.
    #[inline]
    pub fn is_present(&self) -> bool {
        self.data.get().is_some()
    }

    /// Borrows the layout data immutably, *asserting that there are no mutators*. Bad things will
    /// happen if you try to mutate the layout data while this is held. This is the only thread-
    /// safe layout data accessor.
    ///
    /// FIXME(pcwalton): Enforce this invariant via the type system. Will require traversal
    /// functions to be trusted, but c'est la vie.
    #[inline]
    pub unsafe fn borrow_unchecked<'a>(&'a self) -> &'a () {
        cast::transmute(self.data.borrow_unchecked())
    }

    /// Borrows the layout data immutably. This function is *not* thread-safe.
    #[inline]
    pub fn borrow<'a>(&'a self) -> SlotRef<'a,()> {
        unsafe {
            cast::transmute(self.data.borrow())
        }
    }

    /// Borrows the layout data mutably. This function is *not* thread-safe.
    ///
    /// FIXME(pcwalton): We should really put this behind a `MutLayoutView` phantom type, to
    /// prevent CSS selector matching from mutably accessing nodes it's not supposed to and racing
    /// on it. This has already resulted in one bug!
    #[inline]
    pub fn mutate<'a>(&'a self) -> MutSlotRef<'a,()> {
        unsafe {
            cast::transmute(self.data.mutate())
        }
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

impl<View> Clone for AbstractNode<View> {
    fn clone(&self) -> AbstractNode<View> {
        *self
    }
}

impl<View> TreeNodeRef<Node<View>> for AbstractNode<View> {
    fn node<'a>(&'a self) -> &'a Node<View> {
        unsafe {
            &(*self.obj).data
        }
    }

    fn mut_node<'a>(&'a self) -> &'a mut Node<View> {
        unsafe {
            &mut (*self.obj).data
        }
    }

    fn parent_node(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.parent_node
    }
    fn first_child(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.first_child
    }
    fn last_child(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.last_child
    }
    fn prev_sibling(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.prev_sibling
    }
    fn next_sibling(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.next_sibling
    }

    fn set_parent_node(node: &mut Node<View>, new_parent_node: Option<AbstractNode<View>>) {
        let doc = node.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        node.parent_node = new_parent_node
    }
    fn set_first_child(node: &mut Node<View>, new_first_child: Option<AbstractNode<View>>) {
        let doc = node.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        node.first_child = new_first_child
    }
    fn set_last_child(node: &mut Node<View>, new_last_child: Option<AbstractNode<View>>) {
        let doc = node.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        node.last_child = new_last_child
    }
    fn set_prev_sibling(node: &mut Node<View>, new_prev_sibling: Option<AbstractNode<View>>) {
        let doc = node.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        node.prev_sibling = new_prev_sibling
    }
    fn set_next_sibling(node: &mut Node<View>, new_next_sibling: Option<AbstractNode<View>>) {
        let doc = node.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        node.next_sibling = new_next_sibling
    }

    fn is_element(&self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(*) => true,
            _ => false
        }
    }

    fn is_document(&self) -> bool {
        match self.type_id() {
            DocumentNodeTypeId(*) => true,
            _ => false
        }
    }
}

impl<View> TreeNodeRefAsElement<Node<View>, Element> for AbstractNode<View> {
    #[inline]
    fn with_imm_element_like<R>(&self, f: &fn(&Element) -> R) -> R {
        self.with_imm_element(f)
    }
}


impl<View> TreeNode<AbstractNode<View>> for Node<View> { }

impl<'self, View> AbstractNode<View> {
    // Unsafe accessors

    pub unsafe fn as_cacheable_wrapper(&self) -> @mut Reflectable {
        match self.type_id() {
            TextNodeTypeId => {
                let node: @mut Text = cast::transmute(self.obj);
                node as @mut Reflectable
            }
            _ => {
                fail!("unsupported node type")
            }
        }
    }

    /// Allow consumers to recreate an AbstractNode from the raw boxed type.
    /// Must only be used in situations where the boxed type is in the inheritance
    /// chain for nodes.
    pub fn from_box<T>(ptr: *mut Box<T>) -> AbstractNode<View> {
        AbstractNode {
            obj: ptr as *mut Box<Node<View>>
        }
    }

    /// Allow consumers to upcast from derived classes.
    pub fn from_document(doc: AbstractDocument) -> AbstractNode<View> {
        unsafe {
            cast::transmute(doc)
        }
    }

    pub fn from_eventtarget(target: AbstractEventTarget) -> AbstractNode<View> {
        assert!(target.is_node());
        unsafe {
            cast::transmute(target)
        }
    }

    // Convenience accessors

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    pub fn type_id(self) -> NodeTypeId {
        self.node().type_id
    }

    /// Returns the parent node of this node. Fails if this node is borrowed mutably.
    pub fn parent_node(self) -> Option<AbstractNode<View>> {
        self.node().parent_node
    }

    /// Returns the first child of this node. Fails if this node is borrowed mutably.
    pub fn first_child(self) -> Option<AbstractNode<View>> {
        self.node().first_child
    }

    /// Returns the last child of this node. Fails if this node is borrowed mutably.
    pub fn last_child(self) -> Option<AbstractNode<View>> {
        self.node().last_child
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    pub fn prev_sibling(self) -> Option<AbstractNode<View>> {
        self.node().prev_sibling
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    pub fn next_sibling(self) -> Option<AbstractNode<View>> {
        self.node().next_sibling
    }

    //
    // Downcasting borrows
    //

    pub fn transmute<T, R>(self, f: &fn(&T) -> R) -> R {
        unsafe {
            let node_box: *mut Box<Node<View>> = transmute(self.obj);
            let node = &mut (*node_box).data;
            let old = node.abstract;
            node.abstract = Some(self);
            let box: *Box<T> = transmute(self.obj);
            let rv = f(&(*box).data);
            node.abstract = old;
            rv
        }
    }

    pub fn transmute_mut<T, R>(self, f: &fn(&mut T) -> R) -> R {
        unsafe {
            let node_box: *mut Box<Node<View>> = transmute(self.obj);
            let node = &mut (*node_box).data;
            let old = node.abstract;
            node.abstract = Some(self);
            let box: *Box<T> = transmute(self.obj);
            let rv = f(cast::transmute(&(*box).data));
            node.abstract = old;
            rv
        }
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn is_characterdata(self) -> bool {
        // FIXME: ProcessingInstruction
        self.is_text() || self.is_comment()
    }

    pub fn with_imm_characterdata<R>(self, f: &fn(&CharacterData) -> R) -> R {
        if !self.is_characterdata() {
            fail!(~"node is not characterdata");
        }
        self.transmute(f)
    }

    pub fn with_mut_characterdata<R>(self, f: &fn(&mut CharacterData) -> R) -> R {
        if !self.is_characterdata() {
            fail!(~"node is not characterdata");
        }
        self.transmute_mut(f)
    }

    pub fn is_doctype(self) -> bool {
        self.type_id() == DoctypeNodeTypeId
    }

    pub fn with_imm_doctype<R>(self, f: &fn(&DocumentType) -> R) -> R {
        if !self.is_doctype() {
            fail!(~"node is not doctype");
        }
        self.transmute(f)
    }

    pub fn with_mut_doctype<R>(self, f: &fn(&mut DocumentType) -> R) -> R {
        if !self.is_doctype() {
            fail!(~"node is not doctype");
        }
        self.transmute_mut(f)
    }

    #[inline]
    pub fn is_comment(self) -> bool {
        // FIXME(pcwalton): Temporary workaround for the lack of inlining of autogenerated `Eq`
        // implementations in Rust.
        match self.type_id() {
            CommentNodeTypeId => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_text(self) -> bool {
        // FIXME(pcwalton): Temporary workaround for the lack of inlining of autogenerated `Eq`
        // implementations in Rust.
        match self.type_id() {
            TextNodeTypeId => true,
            _ => false,
        }
    }

    pub fn with_imm_text<R>(self, f: &fn(&Text) -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute(f)
    }

    pub fn with_mut_text<R>(self, f: &fn(&mut Text) -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute_mut(f)
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn with_imm_element<R>(self, f: &fn(&Element) -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute(f)
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn as_mut_element<R>(self, f: &fn(&mut Element) -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute_mut(f)
    }

    #[inline]
    pub fn is_image_element(self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => true,
            _ => false,
        }
    }

    pub fn with_imm_image_element<R>(self, f: &fn(&HTMLImageElement) -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute(f)
    }

    pub fn with_mut_image_element<R>(self, f: &fn(&mut HTMLImageElement) -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute_mut(f)
    }

    pub fn is_iframe_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLIframeElementTypeId)
    }

    pub fn with_imm_iframe_element<R>(self, f: &fn(&HTMLIFrameElement) -> R) -> R {
        if !self.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.transmute(f)
    }

    pub fn with_mut_iframe_element<R>(self, f: &fn(&mut HTMLIFrameElement) -> R) -> R {
        if !self.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.transmute_mut(f)
    }

    pub fn is_style_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLStyleElementTypeId)
    }

    pub fn is_anchor_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLAnchorElementTypeId)
    }

    pub unsafe fn raw_object(self) -> *mut Box<Node<View>> {
        self.obj
    }

    pub fn from_raw(raw: *mut Box<Node<View>>) -> AbstractNode<View> {
        AbstractNode {
            obj: raw
        }
    }

    /// Dumps the subtree rooted at this node, for debugging.
    pub fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    pub fn dump_indent(&self, indent: uint) {
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
    pub fn debug_str(&self) -> ~str {
        format!("{:?}", self.type_id())
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator<View> {
        self.node().children()
    }

    // Issue #1030: should not walk the tree
    pub fn is_in_doc(&self) -> bool {
        self.ancestors().any(|node| node.is_document())
    }
}

impl AbstractNode<ScriptView> {
    pub fn AppendChild(self, node: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        self.node().AppendChild(self, node)
    }

    pub fn RemoveChild(self, node: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        self.node().RemoveChild(self, node)
    }

    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(self) {
        assert!(self.parent_node().is_some());
        let document = self.node().owner_doc();

        // Register elements having "id" attribute to the owner doc.
        document.mut_document().register_nodes_with_id(&self);

        document.document().content_changed();
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(self) {
        assert!(self.parent_node().is_none());
        let document = self.node().owner_doc();

        // Unregister elements having "id".
        document.mut_document().unregister_nodes_with_id(&self);

        document.document().content_changed();
    }
}

impl<View> Iterator<AbstractNode<View>> for AbstractNodeChildrenIterator<View> {
    fn next(&mut self) -> Option<AbstractNode<View>> {
        let node = self.current_node;
        self.current_node = do self.current_node.and_then |node| {
            node.next_sibling()
        };
        node
    }
}

impl<View> Node<View> {
    pub fn owner_doc(&self) -> AbstractDocument {
        self.owner_doc.unwrap()
    }

    pub fn set_owner_doc(&mut self, document: AbstractDocument) {
        self.owner_doc = Some(document);
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator<View> {
        AbstractNodeChildrenIterator {
            current_node: self.first_child,
        }
    }
}

impl Node<ScriptView> {
    pub fn reflect_node<N: Reflectable>
            (node:      @mut N,
             document:  AbstractDocument,
             wrap_fn:   extern "Rust" fn(*JSContext, *JSObject, @mut N) -> *JSObject)
             -> AbstractNode<ScriptView> {
        assert!(node.reflector().get_jsobject().is_null());
        let node = reflect_dom_object(node, document.document().window, wrap_fn);
        assert!(node.reflector().get_jsobject().is_not_null());
        // This surrenders memory management of the node!
        AbstractNode {
            obj: unsafe { transmute(node) },
        }
    }

    pub fn new(type_id: NodeTypeId, doc: AbstractDocument) -> Node<ScriptView> {
        Node::new_(type_id, Some(doc))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node<ScriptView> {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<AbstractDocument>) -> Node<ScriptView> {
        Node {
            eventtarget: EventTarget::new_inherited(NodeTypeId),
            type_id: type_id,

            abstract: None,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            owner_doc: doc,
            child_list: None,

            layout_data: LayoutDataRef::init(),
        }
    }

    /// Sends layout data, if any, back to the script task to be destroyed.
    pub unsafe fn reap_layout_data(&mut self) {
        if self.layout_data.is_present() {
            let layout_data = util::replace(&mut self.layout_data, LayoutDataRef::init());
            let js_window = utils::global_object_for_dom_object(self);
            (*js_window).data.page.reap_dead_layout_data(layout_data)
        }
    }
}

impl Node<ScriptView> {
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

    pub fn NodeName(&self, abstract_self: AbstractNode<ScriptView>) -> DOMString {
        match self.type_id {
            ElementNodeTypeId(*) => {
                do abstract_self.with_imm_element |element| {
                    element.TagName()
                }
            }
            CommentNodeTypeId => ~"#comment",
            TextNodeTypeId => ~"#text",
            DoctypeNodeTypeId => {
                do abstract_self.with_imm_doctype |doctype| {
                    doctype.name.clone()
                }
            },
            DocumentFragmentNodeTypeId => ~"#document-fragment",
            DocumentNodeTypeId(_) => ~"#document"
        }
    }

    pub fn GetBaseURI(&self) -> Option<DOMString> {
        None
    }

    pub fn GetOwnerDocument(&self) -> Option<AbstractDocument> {
        match self.type_id {
            ElementNodeTypeId(*) |
            CommentNodeTypeId |
            TextNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc()),
            DocumentNodeTypeId(_) => None
        }
    }

    pub fn GetParentNode(&self) -> Option<AbstractNode<ScriptView>> {
        self.parent_node
    }

    pub fn GetParentElement(&self) -> Option<AbstractNode<ScriptView>> {
        self.parent_node.filtered(|parent| parent.is_element())
    }

    pub fn HasChildNodes(&self) -> bool {
        self.first_child.is_some()
    }

    pub fn GetFirstChild(&self) -> Option<AbstractNode<ScriptView>> {
        self.first_child
    }

    pub fn GetLastChild(&self) -> Option<AbstractNode<ScriptView>> {
        self.last_child
    }

    pub fn GetPreviousSibling(&self) -> Option<AbstractNode<ScriptView>> {
        self.prev_sibling
    }

    pub fn GetNextSibling(&self) -> Option<AbstractNode<ScriptView>> {
        self.next_sibling
    }

    pub fn GetNodeValue(&self, abstract_self: AbstractNode<ScriptView>) -> Option<DOMString> {
        match self.type_id {
            // ProcessingInstruction
            CommentNodeTypeId | TextNodeTypeId => {
                do abstract_self.with_imm_characterdata() |characterdata| {
                    Some(characterdata.Data())
                }
            }
            _ => {
                None
            }
        }
    }

    pub fn SetNodeValue(&mut self, _abstract_self: AbstractNode<ScriptView>, _val: Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn GetTextContent(&self, abstract_self: AbstractNode<ScriptView>) -> Option<DOMString> {
        match self.type_id {
          DocumentFragmentNodeTypeId | ElementNodeTypeId(*) => {
            let mut content = ~"";
            for node in abstract_self.traverse_preorder() {
                if node.is_text() {
                    do node.with_imm_text() |text| {
                        content = content + text.element.Data();
                    }
                }
            }
            Some(content)
          }
          CommentNodeTypeId | TextNodeTypeId => {
            do abstract_self.with_imm_characterdata() |characterdata| {
                Some(characterdata.Data())
            }
          }
          DoctypeNodeTypeId | DocumentNodeTypeId(_) => {
            None
          }
        }
    }

    pub fn ChildNodes(&mut self, abstract_self: AbstractNode<ScriptView>) -> @mut NodeList {
        match self.child_list {
            None => {
                let window = self.owner_doc().document().window;
                let list = NodeList::new_child_list(window, abstract_self);
                self.child_list = Some(list);
                list
            }
            Some(list) => list
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    fn adopt(node: AbstractNode<ScriptView>, document: AbstractDocument) {
        // Step 1.
        match node.parent_node() {
            Some(parent) => Node::remove(node, parent, false),
            None => (),
        }

        // Step 2.
        if node.node().owner_doc() != document {
            for descendant in node.traverse_preorder() {
                descendant.mut_node().set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node: AbstractNode<ScriptView>,
                  parent: AbstractNode<ScriptView>,
                  child: Option<AbstractNode<ScriptView>>) -> Fallible<AbstractNode<ScriptView>> {
        fn is_inclusive_ancestor_of(node: AbstractNode<ScriptView>,
                                    parent: AbstractNode<ScriptView>) -> bool {
            node == parent || parent.ancestors().any(|ancestor| ancestor == node)
        }

        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId(*) |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(*) => (),
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
            DocumentNodeTypeId(*) => return Err(HierarchyRequest),
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
                fn inclusively_followed_by_doctype(child: Option<AbstractNode<ScriptView>>) -> bool{
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
                        match node.children().count(|c| c.is_element()) {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                if parent.children().any(|c| c.is_element()) {
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
                        if parent.children().any(|c| c.is_element()) {
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
                                if parent.children().any(|c| c.is_element()) {
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
        Node::adopt(node, parent.node().owner_doc());

        // Step 10.
        Node::insert(node, parent, referenceChild, false);

        // Step 11.
        return Ok(node)
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: AbstractNode<ScriptView>,
              parent: AbstractNode<ScriptView>,
              child: Option<AbstractNode<ScriptView>>,
              suppress_observers: bool) {
        // XXX assert owner_doc
        // Step 1-3: ranges.
        // Step 4.
        let nodes = match node.type_id() {
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
        for node in nodes.iter() {
            parent.add_child(*node, child);
        }

        // Step 9.
        if !suppress_observers {
            for node in nodes.iter() {
                node.node_inserted();
            }
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-replace-all
    pub fn replace_all(node: Option<AbstractNode<ScriptView>>,
                       parent: AbstractNode<ScriptView>) {
        // Step 1.
        match node {
            Some(node) => Node::adopt(node, parent.node().owner_doc()),
            None => (),
        }

        // Step 2.
        let removedNodes: ~[AbstractNode<ScriptView>] = parent.children().collect();

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
    fn pre_remove(child: AbstractNode<ScriptView>,
                  parent: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
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
    fn remove(node: AbstractNode<ScriptView>,
              parent: AbstractNode<ScriptView>,
              suppress_observers: bool) {
        assert!(node.parent_node() == Some(parent));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node);

        // Step 9.
        if !suppress_observers {
            node.node_removed();
        }
    }

    pub fn SetTextContent(&mut self,
                          abstract_self: AbstractNode<ScriptView>,
                          value: Option<DOMString>) -> ErrorResult {
        let value = null_str_as_empty(&value);
        match self.type_id {
          DocumentFragmentNodeTypeId | ElementNodeTypeId(*) => {
            // Step 1-2.
            let node = if value.len() == 0 {
                None
            } else {
                let document = self.owner_doc();
                Some(document.document().CreateTextNode(document, value))
            };
            // Step 3.
            Node::replace_all(node, abstract_self);
          }
          CommentNodeTypeId | TextNodeTypeId => {
            self.wait_until_safe_to_modify_dom();

            do abstract_self.with_mut_characterdata() |characterdata| {
                characterdata.data = value.clone();

                // Notify the document that the content of this node is different
                let document = self.owner_doc();
                document.document().content_changed();
            }
          }
          DoctypeNodeTypeId | DocumentNodeTypeId(_) => {}
        }
        Ok(())
    }

    pub fn InsertBefore(&self,
                        node: AbstractNode<ScriptView>,
                        child: Option<AbstractNode<ScriptView>>) -> Fallible<AbstractNode<ScriptView>> {
        Node::pre_insert(node, node, child)
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        let document = self.owner_doc();
        document.document().wait_until_safe_to_modify_dom();
    }

    pub fn AppendChild(&self,
                       abstract_self: AbstractNode<ScriptView>,
                       node: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        Node::pre_insert(node, abstract_self, None)
    }

    pub fn ReplaceChild(&mut self, _node: AbstractNode<ScriptView>, _child: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        fail!("stub")
    }

    pub fn RemoveChild(&self,
                       abstract_self: AbstractNode<ScriptView>,
                       node: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        Node::pre_remove(node, abstract_self)
    }

    pub fn Normalize(&mut self) {
    }

    pub fn CloneNode(&self, _deep: bool) -> Fallible<AbstractNode<ScriptView>> {
        fail!("stub")
    }

    pub fn IsEqualNode(&self, _node: Option<AbstractNode<ScriptView>>) -> bool {
        false
    }

    pub fn CompareDocumentPosition(&self, _other: AbstractNode<ScriptView>) -> u16 {
        0
    }

    pub fn Contains(&self, _other: Option<AbstractNode<ScriptView>>) -> bool {
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
}

impl Reflectable for Node<ScriptView> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
    }
}

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, node: AbstractNode<LayoutView>) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune(&self, _node: AbstractNode<LayoutView>) -> bool {
        false
    }
}

impl AbstractNode<LayoutView> {
    /// Traverses the tree in postorder.
    ///
    /// TODO(pcwalton): Offer a parallel version with a compatible API.
    pub fn traverse_postorder<T:PostorderNodeTraversal>(self, traversal: &T) -> bool {
        if traversal.should_prune(self) {
            return true
        }

        let mut opt_kid = self.first_child();
        loop {
            match opt_kid {
                None => break,
                Some(kid) => {
                    if !kid.traverse_postorder(traversal) {
                        return false
                    }
                    opt_kid = kid.next_sibling()
                }
            }
        }

        traversal.process(self)
    }
}
