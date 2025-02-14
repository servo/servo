/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use servo_atoms::Atom;
use style::author_styles::AuthorStyles;
use style::dom::TElement;
use style::shared_lock::SharedRwLockReadGuard;
use style::stylesheets::Stylesheet;
use style::stylist::{CascadeData, Stylist};

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documentorshadowroot::{DocumentOrShadowRoot, StyleSheetInDocument};
use crate::dom::element::Element;
use crate::dom::node::{
    BindContext, Node, NodeDamage, NodeFlags, NodeTraits, ShadowIncluding, UnbindContext,
};
use crate::dom::stylesheetlist::{StyleSheetList, StyleSheetListOwner};
use crate::dom::types::EventTarget;
use crate::dom::virtualmethods::{vtable_for, VirtualMethods};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
use crate::stylesheet_set::StylesheetSetRef;

/// Whether a shadow root hosts an User Agent widget.
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum IsUserAgentWidget {
    No,
    Yes,
}

/// <https://dom.spec.whatwg.org/#interface-shadowroot>
#[dom_struct]
pub(crate) struct ShadowRoot {
    document_fragment: DocumentFragment,
    document_or_shadow_root: DocumentOrShadowRoot,
    document: Dom<Document>,
    host: MutNullableDom<Element>,
    /// List of author styles associated with nodes in this shadow tree.
    #[custom_trace]
    author_styles: DomRefCell<AuthorStyles<StyleSheetInDocument>>,
    stylesheet_list: MutNullableDom<StyleSheetList>,
    window: Dom<Window>,

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-mode>
    mode: ShadowRootMode,

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-slotassignment>
    slot_assignment_mode: SlotAssignmentMode,

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-clonable>
    clonable: bool,
}

impl ShadowRoot {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        host: &Element,
        document: &Document,
        mode: ShadowRootMode,
        slot_assignment_mode: SlotAssignmentMode,
        clonable: bool,
    ) -> ShadowRoot {
        let document_fragment = DocumentFragment::new_inherited(document);
        let node = document_fragment.upcast::<Node>();
        node.set_flag(NodeFlags::IS_IN_SHADOW_TREE, true);
        node.set_flag(
            NodeFlags::IS_CONNECTED,
            host.upcast::<Node>().is_connected(),
        );

        ShadowRoot {
            document_fragment,
            document_or_shadow_root: DocumentOrShadowRoot::new(document.window()),
            document: Dom::from_ref(document),
            host: MutNullableDom::new(Some(host)),
            author_styles: DomRefCell::new(AuthorStyles::new()),
            stylesheet_list: MutNullableDom::new(None),
            window: Dom::from_ref(document.window()),
            mode,
            slot_assignment_mode,
            clonable,
        }
    }

    pub(crate) fn new(
        host: &Element,
        document: &Document,
        mode: ShadowRootMode,
        slot_assignment_mode: SlotAssignmentMode,
        clonable: bool,
    ) -> DomRoot<ShadowRoot> {
        reflect_dom_object(
            Box::new(ShadowRoot::new_inherited(
                host,
                document,
                mode,
                slot_assignment_mode,
                clonable,
            )),
            document.window(),
            CanGc::note(),
        )
    }

    pub(crate) fn detach(&self) {
        self.document.unregister_shadow_root(self);
        let node = self.upcast::<Node>();
        node.set_containing_shadow_root(None);
        Node::complete_remove_subtree(node, &UnbindContext::new(node, None, None, None));
        self.host.set(None);
    }

    pub(crate) fn get_focused_element(&self) -> Option<DomRoot<Element>> {
        //XXX get retargeted focused element
        None
    }

    pub(crate) fn stylesheet_count(&self) -> usize {
        self.author_styles.borrow().stylesheets.len()
    }

    pub(crate) fn stylesheet_at(&self, index: usize) -> Option<DomRoot<CSSStyleSheet>> {
        let stylesheets = &self.author_styles.borrow().stylesheets;

        stylesheets
            .get(index)
            .and_then(|s| s.owner.upcast::<Node>().get_cssom_stylesheet())
    }

    /// Add a stylesheet owned by `owner` to the list of shadow root sheets, in the
    /// correct tree position.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))] // Owner needs to be rooted already necessarily.
    pub(crate) fn add_stylesheet(&self, owner: &Element, sheet: Arc<Stylesheet>) {
        let stylesheets = &mut self.author_styles.borrow_mut().stylesheets;
        let insertion_point = stylesheets
            .iter()
            .find(|sheet_in_shadow| {
                owner
                    .upcast::<Node>()
                    .is_before(sheet_in_shadow.owner.upcast())
            })
            .cloned();
        DocumentOrShadowRoot::add_stylesheet(
            owner,
            StylesheetSetRef::Author(stylesheets),
            sheet,
            insertion_point,
            self.document.style_shared_lock(),
        );
    }

    /// Remove a stylesheet owned by `owner` from the list of shadow root sheets.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))] // Owner needs to be rooted already necessarily.
    pub(crate) fn remove_stylesheet(&self, owner: &Element, s: &Arc<Stylesheet>) {
        DocumentOrShadowRoot::remove_stylesheet(
            owner,
            s,
            StylesheetSetRef::Author(&mut self.author_styles.borrow_mut().stylesheets),
        )
    }

    pub(crate) fn invalidate_stylesheets(&self) {
        self.document.invalidate_shadow_roots_stylesheets();
        self.author_styles.borrow_mut().stylesheets.force_dirty();
        // Mark the host element dirty so a reflow will be performed.
        if let Some(host) = self.host.get() {
            host.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
        }
    }

    /// Remove any existing association between the provided id and any elements
    /// in this shadow tree.
    pub(crate) fn unregister_element_id(&self, to_unregister: &Element, id: Atom) {
        self.document_or_shadow_root.unregister_named_element(
            self.document_fragment.id_map(),
            to_unregister,
            &id,
        );
    }

    /// Associate an element present in this shadow tree with the provided id.
    pub(crate) fn register_element_id(&self, element: &Element, id: Atom) {
        let root = self
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::No)
            .last()
            .unwrap();
        self.document_or_shadow_root.register_named_element(
            self.document_fragment.id_map(),
            element,
            &id,
            root,
        );
    }
}

impl ShadowRootMethods<crate::DomTypeHolder> for ShadowRoot {
    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<DomRoot<Element>> {
        self.document_or_shadow_root
            .get_active_element(self.get_focused_element(), None, None)
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    fn ElementFromPoint(
        &self,
        x: Finite<f64>,
        y: Finite<f64>,
        can_gc: CanGc,
    ) -> Option<DomRoot<Element>> {
        // Return the result of running the retargeting algorithm with context object
        // and the original result as input.
        match self.document_or_shadow_root.element_from_point(
            x,
            y,
            None,
            self.document.has_browsing_context(),
            can_gc,
        ) {
            Some(e) => {
                let retargeted_node = self
                    .upcast::<EventTarget>()
                    .retarget(e.upcast::<EventTarget>());
                retargeted_node.downcast::<Element>().map(DomRoot::from_ref)
            },
            None => None,
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(
        &self,
        x: Finite<f64>,
        y: Finite<f64>,
        can_gc: CanGc,
    ) -> Vec<DomRoot<Element>> {
        // Return the result of running the retargeting algorithm with context object
        // and the original result as input
        let mut elements = Vec::new();
        for e in self
            .document_or_shadow_root
            .elements_from_point(x, y, None, self.document.has_browsing_context(), can_gc)
            .iter()
        {
            let retargeted_node = self
                .upcast::<EventTarget>()
                .retarget(e.upcast::<EventTarget>());
            if let Some(element) = retargeted_node.downcast::<Element>().map(DomRoot::from_ref) {
                elements.push(element);
            }
        }
        elements
    }

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-mode>
    fn Mode(&self) -> ShadowRootMode {
        self.mode
    }

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-clonable>
    fn Clonable(&self) -> bool {
        self.clonable
    }

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-host>
    fn Host(&self) -> DomRoot<Element> {
        let host = self.host.get();
        host.expect("Trying to get host from a detached shadow root")
    }

    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> DomRoot<StyleSheetList> {
        self.stylesheet_list.or_init(|| {
            StyleSheetList::new(
                &self.window,
                StyleSheetListOwner::ShadowRoot(Dom::from_ref(self)),
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-shadowroot-innerhtml>
    fn InnerHTML(&self) -> DOMString {
        // ShadowRoot's innerHTML getter steps are to return the result of running fragment serializing
        // algorithm steps with this and true.
        self.upcast::<Node>().fragment_serialization_algorithm(true)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-shadowroot-innerhtml>
    fn SetInnerHTML(&self, value: DOMString, can_gc: CanGc) {
        // TODO Step 1. Let compliantString be the result of invoking the Get Trusted Type compliant string algorithm
        // with TrustedHTML, this's relevant global object, the given value, "ShadowRoot innerHTML", and "script".
        let compliant_string = value;

        // Step 2. Let context be this's host.
        let context = self.Host();

        // Step 3. Let fragment be the result of invoking the fragment parsing algorithm steps with context and
        // compliantString.
        let Ok(frag) = context.parse_fragment(compliant_string, can_gc) else {
            // NOTE: The spec doesn't strictly tell us to bail out here, but
            // we can't continue if parsing failed
            return;
        };

        // Step 4. Replace all with fragment within this.
        Node::replace_all(Some(frag.upcast()), self.upcast());
    }

    /// <https://dom.spec.whatwg.org/#dom-shadowroot-slotassignment>
    fn SlotAssignment(&self) -> SlotAssignmentMode {
        self.slot_assignment_mode
    }

    // https://dom.spec.whatwg.org/#dom-shadowroot-onslotchange
    event_handler!(onslotchange, GetOnslotchange, SetOnslotchange);
}

impl VirtualMethods for ShadowRoot {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<DocumentFragment>() as &dyn VirtualMethods)
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if context.tree_connected {
            let document = self.owner_document();
            document.register_shadow_root(self);
        }

        let shadow_root = self.upcast::<Node>();

        shadow_root.set_flag(NodeFlags::IS_CONNECTED, context.tree_connected);

        // avoid iterate over the shadow root itself
        for node in shadow_root.traverse_preorder(ShadowIncluding::Yes).skip(1) {
            node.set_flag(NodeFlags::IS_CONNECTED, context.tree_connected);

            // Out-of-document elements never have the descendants flag set
            debug_assert!(!node.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS));
            vtable_for(&node).bind_to_tree(&BindContext {
                tree_connected: context.tree_connected,
                tree_is_in_a_document_tree: false,
                tree_is_in_a_shadow_tree: true,
            });
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_connected {
            let document = self.owner_document();
            document.unregister_shadow_root(self);
        }
    }
}

#[allow(unsafe_code)]
pub(crate) trait LayoutShadowRootHelpers<'dom> {
    fn get_host_for_layout(self) -> LayoutDom<'dom, Element>;
    fn get_style_data_for_layout(self) -> &'dom CascadeData;
    unsafe fn flush_stylesheets<E: TElement>(
        self,
        stylist: &mut Stylist,
        guard: &SharedRwLockReadGuard,
    );
}

impl<'dom> LayoutShadowRootHelpers<'dom> for LayoutDom<'dom, ShadowRoot> {
    #[inline]
    #[allow(unsafe_code)]
    fn get_host_for_layout(self) -> LayoutDom<'dom, Element> {
        unsafe {
            self.unsafe_get()
                .host
                .get_inner_as_layout()
                .expect("We should never do layout on a detached shadow root")
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_style_data_for_layout(self) -> &'dom CascadeData {
        fn is_sync<T: Sync>() {}
        let _ = is_sync::<CascadeData>;
        unsafe { &self.unsafe_get().author_styles.borrow_for_layout().data }
    }

    // FIXME(nox): This uses the dreaded borrow_mut_for_layout so this should
    // probably be revisited.
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn flush_stylesheets<E: TElement>(
        self,
        stylist: &mut Stylist,
        guard: &SharedRwLockReadGuard,
    ) {
        let author_styles = self.unsafe_get().author_styles.borrow_mut_for_layout();
        if author_styles.stylesheets.dirty() {
            author_styles.flush::<E>(stylist, guard);
        }
    }
}

impl Convert<devtools_traits::ShadowRootMode> for ShadowRootMode {
    fn convert(self) -> devtools_traits::ShadowRootMode {
        match self {
            ShadowRootMode::Open => devtools_traits::ShadowRootMode::Open,
            ShadowRootMode::Closed => devtools_traits::ShadowRootMode::Closed,
        }
    }
}
