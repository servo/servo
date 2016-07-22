/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use gecko_bindings::bindings;
use gecko_bindings::bindings::Gecko_ChildrenCount;
use gecko_bindings::bindings::Gecko_ClassOrClassList;
use gecko_bindings::bindings::Gecko_GetNodeData;
use gecko_bindings::bindings::ServoNodeData;
use gecko_bindings::bindings::{Gecko_ElementState, Gecko_GetDocumentElement};
use gecko_bindings::bindings::{Gecko_GetFirstChild, Gecko_GetFirstChildElement};
use gecko_bindings::bindings::{Gecko_GetLastChild, Gecko_GetLastChildElement};
use gecko_bindings::bindings::{Gecko_GetNextSibling, Gecko_GetNextSiblingElement};
use gecko_bindings::bindings::{Gecko_GetNodeFlags, Gecko_SetNodeFlags, Gecko_UnsetNodeFlags};
use gecko_bindings::bindings::{Gecko_GetParentElement, Gecko_GetParentNode};
use gecko_bindings::bindings::{Gecko_GetPrevSibling, Gecko_GetPrevSiblingElement};
use gecko_bindings::bindings::{Gecko_GetServoDeclarationBlock, Gecko_IsHTMLElementInHTMLDocument};
use gecko_bindings::bindings::{Gecko_IsLink, Gecko_IsRootElement, Gecko_IsTextNode};
use gecko_bindings::bindings::{Gecko_IsUnvisitedLink, Gecko_IsVisitedLink};
use gecko_bindings::bindings::{Gecko_LocalName, Gecko_Namespace, Gecko_NodeIsElement, Gecko_SetNodeData};
use gecko_bindings::bindings::{RawGeckoDocument, RawGeckoElement, RawGeckoNode};
use gecko_bindings::structs::nsIAtom;
use gecko_bindings::structs::{NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO, NODE_IS_DIRTY_FOR_SERVO};
use glue::GeckoDeclarationBlock;
use libc::uintptr_t;
use selectors::Element;
use selectors::matching::DeclarationBlock;
use selectors::parser::{AttrSelector, NamespaceConstraint};
use snapshot::GeckoElementSnapshot;
use snapshot_helpers;
use std::marker::PhantomData;
use std::ops::BitOr;
use std::ptr;
use std::sync::Arc;
use string_cache::{Atom, BorrowedAtom, BorrowedNamespace, Namespace};
use style::data::PrivateStyleData;
use style::dom::{OpaqueNode, PresentationalHintsSynthetizer};
use style::dom::{TDocument, TElement, TNode, TRestyleDamage, UnsafeNode};
use style::element_state::ElementState;
use style::error_reporting::StdoutErrorReporter;
use style::gecko_selector_impl::{GeckoSelectorImpl, NonTSPseudoClass};
use style::parser::ParserContextExtraData;
use style::properties::{ComputedValues, parse_style_attribute};
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style::refcell::{Ref, RefCell, RefMut};
use style::selector_impl::ElementExt;
use style::sink::Push;
use url::Url;

pub type NonOpaqueStyleData = *mut RefCell<PrivateStyleData>;

// Important: We don't currently refcount the DOM, because the wrapper lifetime
// magic guarantees that our LayoutFoo references won't outlive the root, and
// we don't mutate any of the references on the Gecko side during restyle. We
// could implement refcounting if need be (at a potentially non-trivial
// performance cost) by implementing Drop and making LayoutFoo non-Copy.
#[derive(Clone, Copy)]
pub struct GeckoNode<'ln> {
    node: *mut RawGeckoNode,
    chain: PhantomData<&'ln ()>,
}

impl<'ln> GeckoNode<'ln> {
    pub unsafe fn from_raw(n: *mut RawGeckoNode) -> GeckoNode<'ln> {
        GeckoNode {
            node: n,
            chain: PhantomData,
        }
    }

    unsafe fn from_ref(n: &RawGeckoNode) -> GeckoNode<'ln> {
        GeckoNode::from_raw(n as *const RawGeckoNode as *mut RawGeckoNode)
    }

    fn get_node_data(&self) -> NonOpaqueStyleData {
        unsafe {
            Gecko_GetNodeData(self.node) as NonOpaqueStyleData
        }
    }

    pub fn initialize_data(self) {
        unsafe {
            if self.get_node_data().is_null() {
                let ptr: NonOpaqueStyleData = Box::into_raw(Box::new(RefCell::new(PrivateStyleData::new())));
                Gecko_SetNodeData(self.node, ptr as *mut ServoNodeData);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct DummyRestyleDamage;
impl TRestyleDamage for DummyRestyleDamage {
    fn compute(_: Option<&Arc<ComputedValues>>, _: &ComputedValues) -> Self { DummyRestyleDamage }
    fn rebuild_and_reflow() -> Self { DummyRestyleDamage }
}
impl BitOr for DummyRestyleDamage {
    type Output = Self;
    fn bitor(self, _: Self) -> Self { DummyRestyleDamage }
}



impl<'ln> TNode for GeckoNode<'ln> {
    type ConcreteDocument = GeckoDocument<'ln>;
    type ConcreteElement = GeckoElement<'ln>;
    type ConcreteRestyleDamage = DummyRestyleDamage;

    fn to_unsafe(&self) -> UnsafeNode {
        (self.node as usize, 0)
    }

    unsafe fn from_unsafe(n: &UnsafeNode) -> Self {
        GeckoNode::from_raw(n.0 as *mut RawGeckoNode)
    }

    fn is_text_node(&self) -> bool {
        unsafe {
            Gecko_IsTextNode(self.node)
        }
    }

    fn is_element(&self) -> bool {
        unsafe {
            Gecko_NodeIsElement(self.node)
        }
    }

    fn dump(self) {
        unimplemented!()
    }

    fn opaque(&self) -> OpaqueNode {
        let ptr: uintptr_t = self.node as uintptr_t;
        OpaqueNode(ptr)
    }

    fn layout_parent_node(self, reflow_root: OpaqueNode) -> Option<GeckoNode<'ln>> {
        if self.opaque() == reflow_root {
            None
        } else {
            self.parent_node()
        }
    }

    fn debug_id(self) -> usize {
        unimplemented!()
    }

    fn children_count(&self) -> u32 {
        unsafe {
            Gecko_ChildrenCount(self.node)
        }
    }

    fn as_element(&self) -> Option<GeckoElement<'ln>> {
        if self.is_element() {
            unsafe { Some(GeckoElement::from_raw(self.node as *mut RawGeckoElement)) }
        } else {
            None
        }
    }

    fn as_document(&self) -> Option<GeckoDocument<'ln>> {
        unimplemented!()
    }

    fn has_changed(&self) -> bool {
        // FIXME(bholley) - Implement this to allow incremental reflows!
        true
    }

    unsafe fn set_changed(&self, _value: bool) {
        unimplemented!()
    }

    fn is_dirty(&self) -> bool {
        // Return true unconditionally if we're not yet styled. This is a hack
        // and should go away soon.
        if unsafe { Gecko_GetNodeData(self.node) }.is_null() {
            return true;
        }

        let flags = unsafe { Gecko_GetNodeFlags(self.node) };
        flags & (NODE_IS_DIRTY_FOR_SERVO as u32) != 0
    }

    unsafe fn set_dirty(&self, value: bool) {
        if value {
            Gecko_SetNodeFlags(self.node, NODE_IS_DIRTY_FOR_SERVO as u32)
        } else {
            Gecko_UnsetNodeFlags(self.node, NODE_IS_DIRTY_FOR_SERVO as u32)
        }
    }

    fn has_dirty_descendants(&self) -> bool {
        // Return true unconditionally if we're not yet styled. This is a hack
        // and should go away soon.
        if unsafe { Gecko_GetNodeData(self.node) }.is_null() {
            return true;
        }
        let flags = unsafe { Gecko_GetNodeFlags(self.node) };
        flags & (NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32) != 0
    }

    unsafe fn set_dirty_descendants(&self, value: bool) {
        if value {
            Gecko_SetNodeFlags(self.node, NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32)
        } else {
            Gecko_UnsetNodeFlags(self.node, NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32)
        }
    }

    fn can_be_fragmented(&self) -> bool {
        // FIXME(SimonSapin): Servo uses this to implement CSS multicol / fragmentation
        // Maybe this isn’t useful for Gecko?
        false
    }

    unsafe fn set_can_be_fragmented(&self, _value: bool) {
        // FIXME(SimonSapin): Servo uses this to implement CSS multicol / fragmentation
        // Maybe this isn’t useful for Gecko?
    }

    #[inline(always)]
    unsafe fn borrow_data_unchecked(&self) -> Option<*const PrivateStyleData> {
        self.get_node_data().as_ref().map(|d| d.as_unsafe_cell().get() as *const PrivateStyleData)
    }

    #[inline(always)]
    fn borrow_data(&self) -> Option<Ref<PrivateStyleData>> {
        unsafe {
            self.get_node_data().as_ref().map(|d| d.borrow())
        }
    }

    #[inline(always)]
    fn mutate_data(&self) -> Option<RefMut<PrivateStyleData>> {
        unsafe {
            self.get_node_data().as_ref().map(|d| d.borrow_mut())
        }
    }

    fn restyle_damage(self) -> Self::ConcreteRestyleDamage { DummyRestyleDamage }

    fn set_restyle_damage(self, _: Self::ConcreteRestyleDamage) {}

    fn parent_node(&self) -> Option<GeckoNode<'ln>> {
        unsafe {
            Gecko_GetParentNode(self.node).as_ref().map(|n| GeckoNode::from_ref(n))
        }
    }

    fn first_child(&self) -> Option<GeckoNode<'ln>> {
        unsafe {
            Gecko_GetFirstChild(self.node).as_ref().map(|n| GeckoNode::from_ref(n))
        }
    }

    fn last_child(&self) -> Option<GeckoNode<'ln>> {
        unsafe {
            Gecko_GetLastChild(self.node).as_ref().map(|n| GeckoNode::from_ref(n))
        }
    }

    fn prev_sibling(&self) -> Option<GeckoNode<'ln>> {
        unsafe {
            Gecko_GetPrevSibling(self.node).as_ref().map(|n| GeckoNode::from_ref(n))
        }
    }

    fn next_sibling(&self) -> Option<GeckoNode<'ln>> {
        unsafe {
            Gecko_GetNextSibling(self.node).as_ref().map(|n| GeckoNode::from_ref(n))
        }
    }

    fn needs_dirty_on_viewport_size_changed(&self) -> bool {
        // Gecko's node doesn't have the DIRTY_ON_VIEWPORT_SIZE_CHANGE flag,
        // so we force them to be dirtied on viewport size change, regardless if
        // they use viewport percentage size or not.
        // TODO(shinglyu): implement this in Gecko: https://github.com/servo/servo/pull/11890
        true
    }

    // TODO(shinglyu): implement this in Gecko: https://github.com/servo/servo/pull/11890
    unsafe fn set_dirty_on_viewport_size_changed(&self) {}
}

#[derive(Clone, Copy)]
pub struct GeckoDocument<'ld> {
    document: *mut RawGeckoDocument,
    chain: PhantomData<&'ld ()>,
}

impl<'ld> GeckoDocument<'ld> {
    pub unsafe fn from_raw(doc: *mut RawGeckoDocument) -> GeckoDocument<'ld> {
        GeckoDocument {
            document: doc,
            chain: PhantomData,
        }
    }
}

impl<'ld> TDocument for GeckoDocument<'ld> {
    type ConcreteNode = GeckoNode<'ld>;
    type ConcreteElement = GeckoElement<'ld>;

    fn as_node(&self) -> GeckoNode<'ld> {
        unsafe { GeckoNode::from_raw(self.document as *mut RawGeckoNode) }
    }

    fn root_node(&self) -> Option<GeckoNode<'ld>> {
        unsafe {
            Gecko_GetDocumentElement(self.document).as_ref().map(|el| GeckoElement::from_ref(el).as_node())
        }
    }

    fn drain_modified_elements(&self) -> Vec<(GeckoElement<'ld>, GeckoElementSnapshot)> {
        unimplemented!()
        /*
        let elements =  unsafe { self.document.drain_modified_elements() };
        elements.into_iter().map(|(el, snapshot)| (ServoLayoutElement::from_layout_js(el), snapshot)).collect()*/
    }
}

#[derive(Clone, Copy)]
pub struct GeckoElement<'le> {
    element: *mut RawGeckoElement,
    chain: PhantomData<&'le ()>,
}

impl<'le> GeckoElement<'le> {
    pub unsafe fn from_raw(el: *mut RawGeckoElement) -> GeckoElement<'le> {
        GeckoElement {
            element: el,
            chain: PhantomData,
        }
    }

    unsafe fn from_ref(el: &RawGeckoElement) -> GeckoElement<'le> {
        GeckoElement::from_raw(el as *const RawGeckoElement as *mut RawGeckoElement)
    }

    pub fn parse_style_attribute(value: &str) -> Option<PropertyDeclarationBlock> {
        // FIXME(bholley): Real base URL and error reporter.
        let base_url = &*DUMMY_BASE_URL;
        // FIXME(heycam): Needs real ParserContextExtraData so that URLs parse
        // properly.
        let extra_data = ParserContextExtraData::default();
        Some(parse_style_attribute(value, &base_url, Box::new(StdoutErrorReporter), extra_data))
    }
}

lazy_static! {
    pub static ref DUMMY_BASE_URL: Url = {
        Url::parse("http://www.example.org").unwrap()
    };
}

static NO_STYLE_ATTRIBUTE: Option<PropertyDeclarationBlock> = None;

impl<'le> TElement for GeckoElement<'le> {
    type ConcreteNode = GeckoNode<'le>;
    type ConcreteDocument = GeckoDocument<'le>;

    fn as_node(&self) -> Self::ConcreteNode {
        unsafe { GeckoNode::from_raw(self.element as *mut RawGeckoNode) }
    }

    fn style_attribute(&self) -> &Option<PropertyDeclarationBlock> {
        unsafe {
            let ptr = Gecko_GetServoDeclarationBlock(self.element) as *mut GeckoDeclarationBlock;
            ptr.as_ref().map(|d| &d.declarations).unwrap_or(&NO_STYLE_ATTRIBUTE)
        }
    }

    fn get_state(&self) -> ElementState {
        unsafe {
            ElementState::from_bits_truncate(Gecko_ElementState(self.element) as u16)
        }
    }

    #[inline]
    fn has_attr(&self, namespace: &Namespace, attr: &Atom) -> bool {
        unsafe {
            bindings::Gecko_HasAttr(self.element,
                                    namespace.0.as_ptr(),
                                    attr.as_ptr())
        }
    }

    #[inline]
    fn attr_equals(&self, namespace: &Namespace, attr: &Atom, val: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrEquals(self.element,
                                       namespace.0.as_ptr(),
                                       attr.as_ptr(),
                                       val.as_ptr(),
                                       /* ignoreCase = */ false)
        }
    }
}

impl<'le> PresentationalHintsSynthetizer for GeckoElement<'le> {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, _hints: &mut V)
        where V: Push<DeclarationBlock<Vec<PropertyDeclaration>>>
    {
        // FIXME(bholley) - Need to implement this.
    }
}

impl<'le> ::selectors::Element for GeckoElement<'le> {
    type Impl = GeckoSelectorImpl;

    fn parent_element(&self) -> Option<Self> {
        unsafe {
            Gecko_GetParentElement(self.element).as_ref().map(|el| GeckoElement::from_ref(el))
        }
    }

    fn first_child_element(&self) -> Option<Self> {
        unsafe {
            Gecko_GetFirstChildElement(self.element).as_ref().map(|el| GeckoElement::from_ref(el))
        }
    }

    fn last_child_element(&self) -> Option<Self> {
        unsafe {
            Gecko_GetLastChildElement(self.element).as_ref().map(|el| GeckoElement::from_ref(el))
        }
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        unsafe {
            Gecko_GetPrevSiblingElement(self.element).as_ref().map(|el| GeckoElement::from_ref(el))
        }
    }

    fn next_sibling_element(&self) -> Option<Self> {
        unsafe {
            Gecko_GetNextSiblingElement(self.element).as_ref().map(|el| GeckoElement::from_ref(el))
        }
    }

    fn is_root(&self) -> bool {
        unsafe {
            Gecko_IsRootElement(self.element)
        }
    }

    fn is_empty(&self) -> bool {
        // XXX(emilio): Implement this properly.
        false
    }

    fn get_local_name<'a>(&'a self) -> BorrowedAtom<'a> {
        unsafe {
            BorrowedAtom::new(Gecko_LocalName(self.element))
        }
    }

    fn get_namespace<'a>(&'a self) -> BorrowedNamespace<'a> {
        unsafe {
            BorrowedNamespace::new(Gecko_Namespace(self.element))
        }
    }

    fn match_non_ts_pseudo_class(&self, pseudo_class: NonTSPseudoClass) -> bool {
        match pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::AnyLink => unsafe { Gecko_IsLink(self.element) },
            NonTSPseudoClass::Link => unsafe { Gecko_IsUnvisitedLink(self.element) },
            NonTSPseudoClass::Visited => unsafe { Gecko_IsVisitedLink(self.element) },
            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::Indeterminate => {
                self.get_state().contains(pseudo_class.state_flag())
            },
            NonTSPseudoClass::ReadOnly => {
                !self.get_state().contains(pseudo_class.state_flag())
            }
        }
    }

    fn get_id(&self) -> Option<Atom> {
        let ptr = unsafe {
            bindings::Gecko_AtomAttrValue(self.element,
                                          atom!("id").as_ptr())
        };

        if ptr.is_null() {
            None
        } else {
            Some(Atom::from(ptr))
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        snapshot_helpers::has_class(self.element,
                                    name,
                                    Gecko_ClassOrClassList)
    }

    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom)
    {
        snapshot_helpers::each_class(self.element,
                                     callback,
                                     Gecko_ClassOrClassList)
    }

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe {
            Gecko_IsHTMLElementInHTMLDocument(self.element)
        }
    }
}

pub trait AttrSelectorHelpers {
    fn ns_or_null(&self) -> *mut nsIAtom;
    fn select_name(&self, is_html_element_in_html_document: bool) -> *mut nsIAtom;
}

impl AttrSelectorHelpers for AttrSelector {
    fn ns_or_null(&self) -> *mut nsIAtom {
        match self.namespace {
            NamespaceConstraint::Any => ptr::null_mut(),
            NamespaceConstraint::Specific(ref ns) => ns.0.as_ptr(),
        }
    }

    fn select_name(&self, is_html_element_in_html_document: bool) -> *mut nsIAtom {
        if is_html_element_in_html_document {
            self.lower_name.as_ptr()
        } else {
            self.name.as_ptr()
        }
    }
}

impl<'le> ::selectors::MatchAttr for GeckoElement<'le> {
    type AttrString = Atom;
    fn match_attr_has(&self, attr: &AttrSelector) -> bool {
        unsafe {
            bindings::Gecko_HasAttr(self.element,
                                    attr.ns_or_null(),
                                    attr.select_name(self.is_html_element_in_html_document()))
        }
    }
    fn match_attr_equals(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrEquals(self.element,
                                       attr.ns_or_null(),
                                       attr.select_name(self.is_html_element_in_html_document()),
                                       value.as_ptr(),
                                       /* ignoreCase = */ false)
        }
    }
    fn match_attr_equals_ignore_ascii_case(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrEquals(self.element,
                                       attr.ns_or_null(),
                                       attr.select_name(self.is_html_element_in_html_document()),
                                       value.as_ptr(),
                                       /* ignoreCase = */ false)
        }
    }
    fn match_attr_includes(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrIncludes(self.element,
                                         attr.ns_or_null(),
                                         attr.select_name(self.is_html_element_in_html_document()),
                                         value.as_ptr())
        }
    }
    fn match_attr_dash(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrDashEquals(self.element,
                                           attr.ns_or_null(),
                                           attr.select_name(self.is_html_element_in_html_document()),
                                           value.as_ptr())
        }
    }
    fn match_attr_prefix(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrHasPrefix(self.element,
                                          attr.ns_or_null(),
                                          attr.select_name(self.is_html_element_in_html_document()),
                                          value.as_ptr())
        }
    }
    fn match_attr_substring(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrHasSubstring(self.element,
                                             attr.ns_or_null(),
                                             attr.select_name(self.is_html_element_in_html_document()),
                                             value.as_ptr())
        }
    }
    fn match_attr_suffix(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_AttrHasSuffix(self.element,
                                          attr.ns_or_null(),
                                          attr.select_name(self.is_html_element_in_html_document()),
                                          value.as_ptr())
        }
    }
}

impl<'le> ElementExt for GeckoElement<'le> {
    type Snapshot = GeckoElementSnapshot;

    #[inline]
    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(NonTSPseudoClass::AnyLink)
    }
}
