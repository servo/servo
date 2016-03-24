/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use bindings::{Gecko_ChildrenCount};
use bindings::{Gecko_ElementState, Gecko_GetAttrAsUTF8, Gecko_GetDocumentElement};
use bindings::{Gecko_GetFirstChild, Gecko_GetFirstChildElement};
use bindings::{Gecko_GetLastChild, Gecko_GetLastChildElement};
use bindings::{Gecko_GetNextSibling, Gecko_GetNextSiblingElement};
use bindings::{Gecko_GetNodeData};
use bindings::{Gecko_GetParentElement, Gecko_GetParentNode};
use bindings::{Gecko_GetPrevSibling, Gecko_GetPrevSiblingElement};
use bindings::{Gecko_IsHTMLElementInHTMLDocument, Gecko_IsLink, Gecko_IsRootElement, Gecko_IsTextNode};
use bindings::{Gecko_IsUnvisitedLink, Gecko_IsVisitedLink};
#[allow(unused_imports)] // Used in commented-out code.
use bindings::{Gecko_LocalName, Gecko_Namespace, Gecko_NodeIsElement, Gecko_SetNodeData};
use bindings::{RawGeckoDocument, RawGeckoElement, RawGeckoNode};
use bindings::{ServoNodeData};
use libc::uintptr_t;
use properties::GeckoComputedValues;
use selector_impl::{GeckoSelectorImpl, NonTSPseudoClass, PrivateStyleData};
use selectors::Element;
use selectors::matching::DeclarationBlock;
use selectors::parser::{AttrSelector, NamespaceConstraint};
use smallvec::VecLike;
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::BitOr;
use std::slice;
use std::str::from_utf8_unchecked;
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use style::dom::{OpaqueNode, TDocument, TElement, TNode, TRestyleDamage, UnsafeNode};
use style::element_state::ElementState;
#[allow(unused_imports)] // Used in commented-out code.
use style::error_reporting::StdoutErrorReporter;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
#[allow(unused_imports)] // Used in commented-out code.
use style::properties::{parse_style_attribute};
use style::restyle_hints::ElementSnapshot;
use style::selector_impl::ElementExt;
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
    unsafe fn from_raw(n: *mut RawGeckoNode) -> GeckoNode<'ln> {
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

    fn initialize_data(self) {
        unsafe {
            if self.get_node_data().is_null() {
                let ptr: NonOpaqueStyleData = Box::into_raw(box RefCell::new(PrivateStyleData::new()));
                Gecko_SetNodeData(self.node, ptr as *mut ServoNodeData);
            }
        }
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
}

impl<'le> TElement for GeckoElement<'le> {
    type ConcreteNode = GeckoNode<'le>;
    type ConcreteDocument = GeckoDocument<'le>;

    fn as_node(&self) -> Self::ConcreteNode {
        unsafe { GeckoNode::from_raw(self.element as *mut RawGeckoNode) }
    }

    fn style_attribute(&self) -> &Option<PropertyDeclarationBlock> {
        panic!("Requires signature modification - only implemented in stylo branch");
        /*
        // FIXME(bholley): We should do what Servo does here. Gecko needs to
        // call into the Servo CSS parser and then cache the resulting block
        // in the nsAttrValue. That will allow us to borrow it from here.
        let attr = self.get_attr(&ns!(), &atom!("style"));
        // FIXME(bholley): Real base URL and error reporter.
        let base_url = Url::parse("http://www.example.org").unwrap();
        attr.map(|v| parse_style_attribute(&v, &base_url, Box::new(StdoutErrorReporter)))
        */
    }

    fn get_state(&self) -> ElementState {
        unsafe {
            ElementState::from_bits_truncate(Gecko_ElementState(self.element))
        }
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, _hints: &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>
    {
        // FIXME(bholley) - Need to implement this.
    }

    #[inline]
    fn get_attr<'a>(&'a self, namespace: &Namespace, name: &Atom) -> Option<&'a str> {
        unsafe {
            let mut length: u32 = 0;
            let ptr = Gecko_GetAttrAsUTF8(self.element, namespace.0.as_ptr(), name.as_ptr(), &mut length);
            reinterpret_string(ptr, length)
        }
    }

    #[inline]
    fn get_attrs<'a>(&'a self, _name: &Atom) -> Vec<&'a str> {
        unimplemented!()
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

    fn get_local_name(&self) -> &Atom {
        panic!("Requires signature modification - only implemented in stylo branch");
        /*
        unsafe {
            let mut length: u32 = 0;
            let p = Gecko_LocalName(self.element, &mut length);
            Atom::from(String::from_utf16(slice::from_raw_parts(p, length as usize)).unwrap())
        }
        */
    }

    fn get_namespace(&self) -> &Namespace {
        panic!("Requires signature modification - only implemented in stylo branch");
        /*
        unsafe {
            let mut length: u32 = 0;
            let p = Gecko_Namespace(self.element, &mut length);
            Namespace(Atom::from(String::from_utf16(slice::from_raw_parts(p, length as usize)).unwrap()))
        }
        */
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
            NonTSPseudoClass::Indeterminate => {
                self.get_state().contains(pseudo_class.state_flag())
            },
        }
    }

    fn get_id(&self) -> Option<Atom> {
        // FIXME(bholley): Servo caches the id atom directly on the element to
        // make this blazing fast. Assuming that was a measured optimization, doing
        // the dumb thing like we do below will almost certainly be a bottleneck.
        self.get_attr(&ns!(), &atom!("id")).map(|s| Atom::from(s))
    }

    fn has_class(&self, _name: &Atom) -> bool {
        unimplemented!()
    }

    fn each_class<F>(&self, mut callback: F) where F: FnMut(&Atom) {
        // FIXME(bholley): Synergize with the DOM to stop splitting strings here.
        if let Some(classes) = self.get_attr(&ns!(), &atom!("class")) {
            for c in classes.split(" ") {
                callback(&Atom::from(c));
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
