/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{self, ShadowRootMode};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::{DocumentOrShadowRoot, StyleSheetInDocument};
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeDamage, NodeFlags};
use crate::dom::stylesheetlist::{StyleSheetList, StyleSheetListOwner};
use crate::dom::window::Window;
use dom_struct::dom_struct;
use selectors::context::QuirksMode;
use servo_arc::Arc;
use style::author_styles::AuthorStyles;
use style::dom::TElement;
use style::media_queries::Device;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheets::Stylesheet;

// https://dom.spec.whatwg.org/#interface-shadowroot
#[dom_struct]
pub struct ShadowRoot {
    document_fragment: DocumentFragment,
    document_or_shadow_root: DocumentOrShadowRoot,
    document: Dom<Document>,
    host: Dom<Element>,
    /// List of author styles associated with nodes in this shadow tree.
    author_styles: DomRefCell<AuthorStyles<StyleSheetInDocument>>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    window: Dom<Window>,
}

impl ShadowRoot {
    #[allow(unrooted_must_root)]
    fn new_inherited(host: &Element, document: &Document) -> ShadowRoot {
        let document_fragment = DocumentFragment::new_inherited(document);
        document_fragment
            .upcast::<Node>()
            .set_flag(NodeFlags::IS_IN_SHADOW_TREE, true);
        ShadowRoot {
            document_fragment,
            document_or_shadow_root: DocumentOrShadowRoot::new(document.window()),
            document: Dom::from_ref(document),
            host: Dom::from_ref(host),
            author_styles: DomRefCell::new(AuthorStyles::new()),
            stylesheet_list: MutNullableDom::new(None),
            window: Dom::from_ref(document.window()),
        }
    }

    pub fn new(host: &Element, document: &Document) -> DomRoot<ShadowRoot> {
        reflect_dom_object(
            Box::new(ShadowRoot::new_inherited(host, document)),
            document.window(),
            ShadowRootBinding::Wrap,
        )
    }

    pub fn get_focused_element(&self) -> Option<DomRoot<Element>> {
        //XXX get retargeted focused element
        None
    }

    pub fn invalidate_stylesheets(&self) {
        self.author_styles.borrow_mut().stylesheets.force_dirty();
        // Mark the host element dirty so a reflow will be performed.
        self.host
            .upcast::<Node>()
            .dirty(NodeDamage::NodeStyleDamaged);
    }
}

impl ShadowRootMethods for ShadowRoot {
    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root
            .get_active_element(self.get_focused_element(), None, None)
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<DomRoot<Element>> {
        // XXX return the result of running the retargeting algorithm with context object
        // and the original result as input
        self.document_or_shadow_root.element_from_point(
            x,
            y,
            None,
            self.document.has_browsing_context(),
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<DomRoot<Element>> {
        // XXX return the result of running the retargeting algorithm with context object
        // and the original result as input
        self.document_or_shadow_root.elements_from_point(
            x,
            y,
            None,
            self.document.has_browsing_context(),
        )
    }

    /// https://dom.spec.whatwg.org/#dom-shadowroot-mode
    fn Mode(&self) -> ShadowRootMode {
        ShadowRootMode::Closed
    }

    /// https://dom.spec.whatwg.org/#dom-shadowroot-host
    fn Host(&self) -> DomRoot<Element> {
        DomRoot::from_ref(&self.host)
    }

    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> DomRoot<StyleSheetList> {
        self.stylesheet_list
            .or_init(|| StyleSheetList::new(&self.window, Box::new(Dom::from_ref(self))))
    }
}

#[allow(unsafe_code)]
pub trait LayoutShadowRootHelpers {
    unsafe fn get_host_for_layout(&self) -> LayoutDom<Element>;
    unsafe fn get_style_data_for_layout<'a, E: TElement>(
        &self,
    ) -> &'a AuthorStyles<StyleSheetInDocument>;
    unsafe fn flush_stylesheets<E: TElement>(
        &self,
        device: &Device,
        quirks_mode: QuirksMode,
        guard: &SharedRwLockReadGuard,
    );
}

impl LayoutShadowRootHelpers for LayoutDom<ShadowRoot> {
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn get_host_for_layout(&self) -> LayoutDom<Element> {
        (*self.unsafe_get()).host.to_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn get_style_data_for_layout<'a, E: TElement>(
        &self,
    ) -> &'a AuthorStyles<StyleSheetInDocument> {
        (*self.unsafe_get()).author_styles.borrow_for_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn flush_stylesheets<E: TElement>(
        &self,
        device: &Device,
        quirks_mode: QuirksMode,
        guard: &SharedRwLockReadGuard,
    ) {
        let mut author_styles = (*self.unsafe_get()).author_styles.borrow_mut_for_layout();
        if author_styles.stylesheets.dirty() {
            author_styles.flush::<E>(device, quirks_mode, guard);
        }
    }
}

impl StyleSheetListOwner for Dom<ShadowRoot> {
    fn stylesheet_count(&self) -> usize {
        self.author_styles.borrow().stylesheets.len()
    }

    fn stylesheet_at(&self, index: usize) -> Option<DomRoot<CSSStyleSheet>> {
        let stylesheets = &self.author_styles.borrow().stylesheets;

        stylesheets
            .get(index)
            .and_then(|s| s.owner.upcast::<Node>().get_cssom_stylesheet())
    }

    /// Add a stylesheet owned by `owner` to the list of shadow root sheets, in the
    /// correct tree position.
    #[allow(unrooted_must_root)] // Owner needs to be rooted already necessarily.
    fn add_stylesheet(&self, owner: &Element, sheet: Arc<Stylesheet>) {
        let stylesheets = &mut self.author_styles.borrow_mut().stylesheets;
        let insertion_point = stylesheets
            .iter()
            .find(|sheet_in_shadow| {
                owner
                    .upcast::<Node>()
                    .is_before(sheet_in_shadow.owner.upcast())
            })
            .cloned();
        self.document_or_shadow_root.add_stylesheet(
            owner,
            stylesheets,
            sheet,
            insertion_point,
            self.document.style_shared_lock(),
        );
    }

    /// Remove a stylesheet owned by `owner` from the list of shadow root sheets.
    #[allow(unrooted_must_root)] // Owner needs to be rooted already necessarily.
    fn remove_stylesheet(&self, owner: &Element, s: &Arc<Stylesheet>) {
        self.document_or_shadow_root.remove_stylesheet(
            owner,
            s,
            &mut self.author_styles.borrow_mut().stylesheets,
        )
    }
}
