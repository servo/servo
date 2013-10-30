/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::bindings::node;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::{DOMString, null_str_as_empty};
use dom::bindings::utils::{ErrorResult, Fallible, NotFound, HierarchyRequest};
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementTypeId, HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::element::{HTMLStyleElementTypeId};
use dom::nodelist::{NodeList};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::text::Text;

use std::cast;
use std::cast::transmute;
use std::unstable::raw::Box;
use extra::arc::Arc;
use js::jsapi::{JSObject, JSContext};
use style::ComputedValues;
use style::properties::PropertyDeclaration;
use servo_util::tree::{TreeNode, TreeNodeRef, TreeNodeRefAsElement};
use servo_util::range::Range;
use gfx::display_list::DisplayList;

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
    reflector_: Reflector,

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
    priv owner_doc: AbstractDocument,

    /// The live list of children return by .childNodes.
    child_list: Option<@mut NodeList>,

    /// Layout information. Only the layout task may touch this data.
    priv layout_data: LayoutData,
}

/// The different types of nodes.
#[deriving(Eq)]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    DocumentFragmentNodeTypeId,
    CommentNodeTypeId,
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
        node.parent_node = new_parent_node
    }
    fn set_first_child(node: &mut Node<View>, new_first_child: Option<AbstractNode<View>>) {
        node.first_child = new_first_child
    }
    fn set_last_child(node: &mut Node<View>, new_last_child: Option<AbstractNode<View>>) {
        node.last_child = new_last_child
    }
    fn set_prev_sibling(node: &mut Node<View>, new_prev_sibling: Option<AbstractNode<View>>) {
        node.prev_sibling = new_prev_sibling
    }
    fn set_next_sibling(node: &mut Node<View>, new_next_sibling: Option<AbstractNode<View>>) {
        node.next_sibling = new_next_sibling
    }

    fn is_element(&self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(*) => true,
            _ => false
        }
    }

    fn is_root(&self) -> bool {
        self.parent_node().is_none()
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

    /// Is this node a root?
    pub fn is_root(self) -> bool {
        self.parent_node().is_none()
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

    pub fn is_comment(self) -> bool {
        self.type_id() == CommentNodeTypeId
    }

    pub fn is_text(self) -> bool {
        self.type_id() == TextNodeTypeId
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

    pub fn is_image_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLImageElementTypeId)
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
        debug!("%s", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1u)
        }
    }

    /// Returns a string that describes this node.
    pub fn debug_str(&self) -> ~str {
        fmt!("%?", self.type_id())
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator<View> {
        AbstractNodeChildrenIterator {
            current_node: self.first_child(),
        }
    }

    // Issue #1030: should not walk the tree
    pub fn is_in_doc(&self) -> bool {
        let document = self.node().owner_doc();
        match document.document().GetDocumentElement() {
            None => false,
            Some(root) => {
                let mut node = *self;
                let mut in_doc;
                loop {
                    match node.parent_node() {
                        Some(parent) => {
                            node = parent;
                        },
                        None => {
                            // Issue #1029: this is horrible.
                            in_doc = unsafe { node.raw_object() as uint == root.raw_object() as uint };
                            break;
                        }
                    }
                }
                in_doc
            }
        }
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
        self.owner_doc
    }

    pub fn set_owner_doc(&mut self, document: AbstractDocument) {
        self.owner_doc = document;
    }
}

impl Node<ScriptView> {
    pub unsafe fn as_abstract_node<N>(cx: *JSContext, node: @N) -> AbstractNode<ScriptView> {
        // This surrenders memory management of the node!
        let mut node = AbstractNode {
            obj: transmute(node),
        };
        node::create(cx, &mut node);
        node
    }

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

    pub fn add_to_doc(&mut self, abstract_self: AbstractNode<ScriptView>, doc: AbstractDocument) {
        let old_doc = self.owner_doc();
        self.set_owner_doc(doc);
        let mut cur_node = self.first_child;
        while cur_node.is_some() {
            for node in cur_node.unwrap().descendants_and_self() {
                node.mut_node().set_owner_doc(doc);
            };
            cur_node = cur_node.unwrap().next_sibling();
        }

        // Unregister elements having "id' from the old doc.
        old_doc.mut_document().unregister_nodes_with_id(&abstract_self);

        // Register elements having "id" attribute to the owner doc.
        doc.mut_document().register_nodes_with_id(&abstract_self);

        // Signal the old document that it needs to update its display
        if old_doc != doc {
            old_doc.document().content_changed();
        }

        // Signal the new document that it needs to update its display
        doc.document().content_changed();
    }

    pub fn new(type_id: NodeTypeId, doc: AbstractDocument) -> Node<ScriptView> {
        Node {
            reflector_: Reflector::new(),
            type_id: type_id,

            abstract: None,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            owner_doc: doc,
            child_list: None,

            layout_data: LayoutData::new(),
        }
    }

    pub fn getNextSibling(&mut self) -> Option<&mut AbstractNode<ScriptView>> {
        match self.next_sibling {
            // transmute because the compiler can't deduce that the reference
            // is safe outside of with_mut_base blocks.
            Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
            None => None
        }
    }

    pub fn getFirstChild(&mut self) -> Option<&mut AbstractNode<ScriptView>> {
        match self.first_child {
            // transmute because the compiler can't deduce that the reference
            // is safe outside of with_mut_base blocks.
            Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
            None => None
        }
    }
}

impl Node<ScriptView> {
    pub fn NodeType(&self) -> u16 {
        match self.type_id {
            ElementNodeTypeId(_) => 1,
            TextNodeTypeId       => 3,
            CommentNodeTypeId    => 8,
            DoctypeNodeTypeId    => 10,
            DocumentFragmentNodeTypeId => 11,
        }
    }

    pub fn NodeName(&self, abstract_self: AbstractNode<ScriptView>) -> DOMString {
        Some(match self.type_id {
            ElementNodeTypeId(*) => {
                do abstract_self.with_imm_element |element| {
                    element.TagName().expect("tagName should never be null")
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
        })
    }

    pub fn GetBaseURI(&self) -> DOMString {
        None
    }

    pub fn GetOwnerDocument(&self) -> Option<AbstractDocument> {
        match self.type_id {
            ElementNodeTypeId(*) |
            CommentNodeTypeId |
            TextNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc()),
            // DocumentNodeTypeId => None
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

    pub fn GetNodeValue(&self, abstract_self: AbstractNode<ScriptView>) -> DOMString {
        match self.type_id {
            // ProcessingInstruction
            CommentNodeTypeId | TextNodeTypeId => {
                do abstract_self.with_imm_characterdata() |characterdata| {
                    characterdata.Data()
                }
            }
            _ => {
                None
            }
        }
    }

    pub fn SetNodeValue(&mut self, _abstract_self: AbstractNode<ScriptView>, _val: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetTextContent(&self, abstract_self: AbstractNode<ScriptView>) -> DOMString {
        match self.type_id {
          DocumentFragmentNodeTypeId | ElementNodeTypeId(*) => {
            let mut content = ~"";
            for node in abstract_self.descendants_and_self() {
                if node.is_text() {
                    do node.with_imm_text() |text| {
                        let s = text.element.Data();
                        content = content + null_str_as_empty(&s);
                    }
                }
            }
            Some(content)
          }
          CommentNodeTypeId | TextNodeTypeId => {
            do abstract_self.with_imm_characterdata() |characterdata| {
                characterdata.Data()
            }
          }
          DoctypeNodeTypeId => {
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

    // http://dom.spec.whatwg.org/#concept-node-replace-all
    pub fn replace_all(&mut self,
                       abstract_self: AbstractNode<ScriptView>,
                       node: Option<AbstractNode<ScriptView>>) {
        //FIXME: We should batch document notifications that occur here
        for child in abstract_self.children() {
            self.RemoveChild(abstract_self, child);
        }
        match node {
            None => {},
            Some(node) => {
                self.AppendChild(abstract_self, node);
            }
        }
    }

    pub fn SetTextContent(&mut self,
                          abstract_self: AbstractNode<ScriptView>,
                          value: &DOMString) -> ErrorResult {
        let is_empty = match value {
            &Some(~"") | &None => true,
            _ => false
        };
        match self.type_id {
          DocumentFragmentNodeTypeId | ElementNodeTypeId(*) => {
            let node = if is_empty {
                None
            } else {
                let document = self.owner_doc();
                Some(document.document().CreateTextNode(document, value))
            };
            self.replace_all(abstract_self, node);
          }
          CommentNodeTypeId | TextNodeTypeId => {
            self.wait_until_safe_to_modify_dom();

            do abstract_self.with_mut_characterdata() |characterdata| {
                characterdata.data = null_str_as_empty(value);

                // Notify the document that the content of this node is different
                let document = self.owner_doc();
                document.document().content_changed();
            }
          }
          DoctypeNodeTypeId => {}
        }
        Ok(())
    }

    pub fn InsertBefore(&mut self, _node: AbstractNode<ScriptView>, _child: Option<AbstractNode<ScriptView>>) -> Fallible<AbstractNode<ScriptView>> {
        fail!("stub")
    }

    fn wait_until_safe_to_modify_dom(&self) {
        let document = self.owner_doc();
        document.document().wait_until_safe_to_modify_dom();
    }

    pub fn AppendChild(&mut self,
                       abstract_self: AbstractNode<ScriptView>,
                       node: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        fn is_hierarchy_request_err(this_node: AbstractNode<ScriptView>,
                                   new_child: AbstractNode<ScriptView>) -> bool {
            if new_child.is_doctype() {
                return true;
            }
            if !this_node.is_element() {
                // FIXME: This should also work for Document and DocumentFragments when they inherit from node.
                // per jgraham
                return true;
            }
            if this_node == new_child {
                return true;
            }
            for ancestor in this_node.ancestors() {
                if ancestor == new_child {
                    return true;
                }
            }
            false
        }

        if is_hierarchy_request_err(abstract_self, node) {
            return Err(HierarchyRequest);
        }

        // TODO: Should we handle WRONG_DOCUMENT_ERR here?

        self.wait_until_safe_to_modify_dom();

        // If the node already exists it is removed from current parent node.
        node.parent_node().map(|parent| parent.remove_child(node));
        abstract_self.add_child(node);
        node.mut_node().add_to_doc(node, self.owner_doc());
        Ok(node)
    }

    pub fn ReplaceChild(&mut self, _node: AbstractNode<ScriptView>, _child: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        fail!("stub")
    }

    pub fn RemoveChild(&mut self,
                       abstract_self: AbstractNode<ScriptView>,
                       node: AbstractNode<ScriptView>) -> Fallible<AbstractNode<ScriptView>> {
        fn is_not_found_err(this_node: AbstractNode<ScriptView>,
                            old_child: AbstractNode<ScriptView>) -> bool {
            match old_child.parent_node() {
                Some(parent) if parent == this_node => false,
                _ => true
            }
        }

        if is_not_found_err(abstract_self, node) {
            return Err(NotFound);
        }

        self.wait_until_safe_to_modify_dom();

        // Unregister elements having "id' from the owner doc.
        // This need be called before target nodes are removed from tree.
        self.owner_doc.mut_document().unregister_nodes_with_id(&abstract_self);

        abstract_self.remove_child(node);
        // Signal the document that it needs to update its display.
        let document = self.owner_doc();
        document.document().content_changed();
        Ok(node)
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

    pub fn LookupPrefix(&self, _prefix: &DOMString) -> DOMString {
        None
    }

    pub fn LookupNamespaceURI(&self, _namespace: &DOMString) -> DOMString {
        None
    }

    pub fn IsDefaultNamespace(&self, _namespace: &DOMString) -> bool {
        false
    }

    pub fn GetNamespaceURI(&self) -> DOMString {
        None
    }

    pub fn GetPrefix(&self) -> DOMString {
        None
    }

    pub fn GetLocalName(&self) -> DOMString {
        None
    }

    pub fn HasAttributes(&self) -> bool {
        false
    }
}

impl Reflectable for Node<ScriptView> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }

    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut Reflectable> {
        match self.parent_node {
            Some(node) => Some(unsafe {node.as_cacheable_wrapper()}),
            None => None
        }
    }
}

// This stuff is notionally private to layout, but we put it here because it needs
// to be stored in a Node, and we can't have cross-crate cyclic dependencies.

pub struct DisplayBoxes {
    display_list: Option<Arc<DisplayList<AbstractNode<()>>>>,
    range: Option<Range>,
}

/// Data that layout associates with a node.
pub struct LayoutData {
    /// The results of CSS matching for this node.
    applicable_declarations: ~[Arc<~[PropertyDeclaration]>],

    /// The results of CSS styling for this node.
    style: Option<ComputedValues>,

    /// Description of how to account for recent style changes.
    restyle_damage: Option<int>,

    /// The boxes associated with this node. Used for `getBoundingClientRect()` and friends.
    boxes: DisplayBoxes,
}

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        LayoutData {
            applicable_declarations: ~[],
            style: None,
            restyle_damage: None,
            boxes: DisplayBoxes {
                display_list: None,
                range: None,
            },
        }
    }
}

// This serves as a static assertion that layout data remains sendable. If this is not done, then
// we can have memory unsafety, which usually manifests as shutdown crashes.
fn assert_is_sendable<T:Send>(_: T) {}
fn assert_layout_data_is_sendable() {
    assert_is_sendable(LayoutData::new())
}

impl AbstractNode<LayoutView> {
    // These accessors take a continuation rather than returning a reference, because
    // an AbstractNode doesn't have a lifetime parameter relating to the underlying
    // Node.  Also this makes it easier to switch to RWArc if we decide that is
    // necessary.
    pub fn read_layout_data<R>(self, blk: &fn(data: &LayoutData) -> R) -> R {
        blk(&self.node().layout_data)
    }

    pub fn write_layout_data<R>(self, blk: &fn(data: &mut LayoutData) -> R) -> R {
        blk(&mut self.mut_node().layout_data)
    }
}
