/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use gecko_bindings::bindings::Gecko_ChildrenCount;
use gecko_bindings::bindings::Gecko_ClassOrClassList;
use gecko_bindings::bindings::Gecko_GetElementId;
use gecko_bindings::bindings::Gecko_GetNodeData;
use gecko_bindings::bindings::ServoNodeData;
use gecko_bindings::bindings::{Gecko_ElementState, Gecko_GetAttrAsUTF8, Gecko_GetDocumentElement};
use gecko_bindings::bindings::{Gecko_GetFirstChild, Gecko_GetFirstChildElement};
use gecko_bindings::bindings::{Gecko_GetLastChild, Gecko_GetLastChildElement};
use gecko_bindings::bindings::{Gecko_GetNextSibling, Gecko_GetNextSiblingElement};
use gecko_bindings::bindings::{Gecko_GetParentElement, Gecko_GetParentNode};
use gecko_bindings::bindings::{Gecko_GetPrevSibling, Gecko_GetPrevSiblingElement};
use gecko_bindings::bindings::{Gecko_GetServoDeclarationBlock, Gecko_IsHTMLElementInHTMLDocument};
use gecko_bindings::bindings::{Gecko_IsLink, Gecko_IsRootElement, Gecko_IsTextNode};
use gecko_bindings::bindings::{Gecko_IsUnvisitedLink, Gecko_IsVisitedLink};
#[allow(unused_imports)] // Used in commented-out code.
use gecko_bindings::bindings::{Gecko_LocalName, Gecko_Namespace, Gecko_NodeIsElement, Gecko_SetNodeData};
use gecko_bindings::bindings::{RawGeckoDocument, RawGeckoElement, RawGeckoNode};
use gecko_bindings::structs::nsIAtom;
use glue::GeckoDeclarationBlock;
use libc::uintptr_t;
use properties::GeckoComputedValues;
use selector_impl::{GeckoSelectorImpl, NonTSPseudoClass, PrivateStyleData};
use selectors::Element;
use selectors::matching::DeclarationBlock;
use selectors::parser::{AttrSelector, NamespaceConstraint};
use smallvec::VecLike;
use std::marker::PhantomData;
use std::ops::BitOr;
use std::ptr;
use std::slice;
use std::str::from_utf8_unchecked;
use std::sync::Arc;
use string_cache::{Atom, BorrowedAtom, BorrowedNamespace, Namespace};
use style::dom::{OpaqueNode, PresentationalHintsSynthetizer};
use style::dom::{TDocument, TElement, TNode, TRestyleDamage, UnsafeNode};
use style::element_state::ElementState;
#[allow(unused_imports)] // Used in commented-out code.
use style::error_reporting::StdoutErrorReporter;
#[allow(unused_imports)] // Used in commented-out code.
use style::parser::ParserContextExtraData;
#[allow(unused_imports)] // Used in commented-out code.
use style::properties::parse_style_attribute;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style::refcell::{Ref, RefCell, RefMut};
use style::restyle_hints::ElementSnapshot;
use style::selector_impl::ElementExt;
use style::sink::Push;
#[allow(unused_imports)] // Used in commented-out code.
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
    type ConcreteComputedValues = GeckoComputedValues;
    fn compute(_: Option<&Arc<GeckoComputedValues>>, _: &GeckoComputedValues) -> Self { DummyRestyleDamage }
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
    type ConcreteComputedValues = GeckoComputedValues;

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
        // FIXME(bholley)
        true
    }

    unsafe fn set_dirty(&self, _value: bool) {
        unimplemented!()
    }

    fn has_dirty_descendants(&self) -> bool {
        // FIXME(bholley)
        true
    }

    unsafe fn set_dirty_descendants(&self, _value: bool) {
        unimplemented!()
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

    fn drain_modified_elements(&self) -> Vec<(GeckoElement<'ld>, ElementSnapshot)> {
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
    fn get_attr<'a>(&'a self, namespace: &Namespace, name: &Atom) -> Option<&'a str> {
        unsafe {
            let mut length: u32 = 0;
            let ptr = Gecko_GetAttrAsUTF8(self.element, namespace.0.as_ptr(), name.as_ptr(),
                                          &mut length);
            reinterpret_string(ptr, length)
        }
    }

    #[inline]
    fn get_attrs<'a>(&'a self, _name: &Atom) -> Vec<&'a str> {
        unimplemented!()
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
        unimplemented!()
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
        unsafe {
            let ptr = Gecko_GetElementId(self.element);
            if ptr.is_null() {
                None
            } else {
                Some(Atom::from(ptr))
            }
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        unsafe {
            let mut class: *mut nsIAtom = ptr::null_mut();
            let mut list: *mut *mut nsIAtom = ptr::null_mut();
            let length = Gecko_ClassOrClassList(self.element, &mut class, &mut list);
            match length {
                0 => false,
                1 => name.as_ptr() == class,
                n => {
                    let classes = slice::from_raw_parts(list, n as usize);
                    classes.iter().any(|ptr| name.as_ptr() == *ptr)
                }
            }
        }
    }

    fn each_class<F>(&self, mut callback: F) where F: FnMut(&Atom) {
        unsafe {
            let mut class: *mut nsIAtom = ptr::null_mut();
            let mut list: *mut *mut nsIAtom = ptr::null_mut();
            let length = Gecko_ClassOrClassList(self.element, &mut class, &mut list);
            match length {
                0 => {}
                1 => Atom::with(class, &mut callback),
                n => {
                    let classes = slice::from_raw_parts(list, n as usize);
                    for c in classes {
                        Atom::with(*c, &mut callback)
                    }
                }
            }
        }
    }

    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        // FIXME(bholley): This is copy-pasted from the servo wrapper's version.
        // We should find a way to share it.
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attr(ns, name).map_or(false, |attr| test(attr))
            },
            NamespaceConstraint::Any => {
                self.get_attrs(name).iter().any(|attr| test(*attr))
            }
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe {
            Gecko_IsHTMLElementInHTMLDocument(self.element)
        }
    }
}

impl<'le> ElementExt for GeckoElement<'le> {
    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(NonTSPseudoClass::AnyLink)
    }
}

unsafe fn reinterpret_string<'a>(ptr: *const ::libc::c_char, length: u32) -> Option<&'a str> {
    (ptr as *const u8).as_ref().map(|p| from_utf8_unchecked(slice::from_raw_parts(p, length as usize)))
}
