/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use script_bindings::root::Dom;
use servo_arc::Arc;
use style::media_queries::MediaList as StyleMediaList;
use style::stylesheets::{
    AllowImportRules, Origin, Stylesheet, StylesheetContents, StylesheetInDocument, UrlExtraData,
};

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLStyleElementBinding::HTMLStyleElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::{CspReporting, InlineCheckType};
use crate::dom::css::cssstylesheet::CSSStyleSheet;
use crate::dom::css::stylesheet::StyleSheet as DOMStyleSheet;
use crate::dom::css::stylesheetcontentscache::{
    StylesheetContentsCache, StylesheetContentsCacheKey,
};
use crate::dom::document::Document;
use crate::dom::documentorshadowroot::StylesheetSource;
use crate::dom::element::{AttributeMutation, Element, ElementCreator};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::medialist::MediaList;
use crate::dom::node::{BindContext, ChildrenMutation, Node, NodeTraits, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;
use crate::stylesheet_loader::{ElementStylesheetLoader, StylesheetOwner};

#[dom_struct]
pub(crate) struct HTMLStyleElement {
    htmlelement: HTMLElement,
    #[conditional_malloc_size_of]
    #[no_trace]
    stylesheet: DomRefCell<Option<Arc<Stylesheet>>>,
    #[no_trace]
    stylesheetcontents_cache_key: DomRefCell<Option<StylesheetContentsCacheKey>>,
    cssom_stylesheet: MutNullableDom<CSSStyleSheet>,
    /// <https://html.spec.whatwg.org/multipage/#a-style-sheet-that-is-blocking-scripts>
    parser_inserted: Cell<bool>,
    in_stack_of_open_elements: Cell<bool>,
    pending_loads: Cell<u32>,
    any_failed_load: Cell<bool>,
}

impl HTMLStyleElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        creator: ElementCreator,
    ) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            stylesheet: DomRefCell::new(None),
            stylesheetcontents_cache_key: DomRefCell::new(None),
            cssom_stylesheet: MutNullableDom::new(None),
            parser_inserted: Cell::new(creator.is_parser_created()),
            in_stack_of_open_elements: Cell::new(creator.is_parser_created()),
            pending_loads: Cell::new(0),
            any_failed_load: Cell::new(false),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
        can_gc: CanGc,
    ) -> DomRoot<HTMLStyleElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLStyleElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
            can_gc,
        )
    }

    #[inline]
    fn create_media_list(&self, mq_str: &str) -> StyleMediaList {
        MediaList::parse_media_list(mq_str, &self.owner_window())
    }

    pub(crate) fn parse_own_css(&self) {
        let node = self.upcast::<Node>();
        assert!(
            node.is_in_a_document_tree() || node.is_in_a_shadow_tree(),
            "This stylesheet does not have an owner, so there's no reason to parse its contents"
        );

        // Step 4. of <https://html.spec.whatwg.org/multipage/#the-style-element%3Aupdate-a-style-block>
        let mut type_attribute = self.Type();
        type_attribute.make_ascii_lowercase();
        if !type_attribute.is_empty() && type_attribute != "text/css" {
            return;
        }

        let doc = self.owner_document();
        let global = &self.owner_global();

        // Step 5: If the Should element's inline behavior be blocked by Content Security Policy? algorithm
        // returns "Blocked" when executed upon the style element, "style",
        // and the style element's child text content, then return. [CSP]
        if global
            .get_csp_list()
            .should_elements_inline_type_behavior_be_blocked(
                global,
                self.upcast(),
                InlineCheckType::Style,
                &node.child_text_content().str(),
            )
        {
            return;
        }

        let window = node.owner_window();
        let data = node
            .GetTextContent()
            .expect("Element.textContent must be a string");
        let shared_lock = node.owner_doc().style_shared_lock().clone();
        let mq = Arc::new(shared_lock.wrap(self.create_media_list(&self.Media().str())));
        let loader = ElementStylesheetLoader::new(self.upcast());

        let stylesheetcontents_create_callback = || {
            #[cfg(feature = "tracing")]
            let _span = tracing::trace_span!("ParseStylesheet", servo_profiling = true).entered();
            StylesheetContents::from_str(
                &data.str(),
                UrlExtraData(window.get_url().get_arc()),
                Origin::Author,
                &shared_lock,
                Some(&loader),
                window.css_error_reporter(),
                doc.quirks_mode(),
                AllowImportRules::Yes,
                /* sanitized_output = */ None,
            )
        };

        // For duplicate style sheets with identical content, `StylesheetContents` can be reused
        // to avoid reedundant parsing of the style sheets. Additionally, the cache hit rate of
        // stylo's `CascadeDataCache` can now be significantly improved. When shared `StylesheetContents`
        // is modified, copy-on-write will occur, see `CSSStyleSheet::will_modify`.
        let (cache_key, contents) = StylesheetContentsCache::get_or_insert_with(
            &data.str(),
            &shared_lock,
            UrlExtraData(window.get_url().get_arc()),
            doc.quirks_mode(),
            stylesheetcontents_create_callback,
        );

        let sheet = Arc::new(Stylesheet {
            contents: shared_lock.wrap(contents),
            shared_lock,
            media: mq,
            disabled: AtomicBool::new(false),
        });

        // No subresource loads were triggered, queue load event
        if self.pending_loads.get() == 0 {
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue_simple_event(self.upcast(), atom!("load"));
        }

        self.set_stylesheet(sheet, cache_key, true);
    }

    // FIXME(emilio): This is duplicated with HTMLLinkElement::set_stylesheet.
    //
    // With the reuse of `StylesheetContent` for same stylesheet string content,
    // this function has a bit difference with `HTMLLinkElement::set_stylesheet` now.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_stylesheet(
        &self,
        s: Arc<Stylesheet>,
        cache_key: Option<StylesheetContentsCacheKey>,
        need_clean_cssom: bool,
    ) {
        let stylesheets_owner = self.stylesheet_list_owner();
        if let Some(ref s) = *self.stylesheet.borrow() {
            stylesheets_owner
                .remove_stylesheet(StylesheetSource::Element(Dom::from_ref(self.upcast())), s);
        }

        if need_clean_cssom {
            self.clean_stylesheet_ownership();
        } else if let Some(cssom_stylesheet) = self.cssom_stylesheet.get() {
            let guard = s.shared_lock.read();
            cssom_stylesheet.update_style_stylesheet(&s, &guard);
        }

        *self.stylesheet.borrow_mut() = Some(s.clone());
        *self.stylesheetcontents_cache_key.borrow_mut() = cache_key;
        stylesheets_owner.add_owned_stylesheet(self.upcast(), s);
    }

    pub(crate) fn will_modify_stylesheet(&self) {
        if let Some(stylesheet_with_owned_contents) = self.create_owned_contents_stylesheet() {
            self.set_stylesheet(stylesheet_with_owned_contents, None, false);
        }
    }

    pub(crate) fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub(crate) fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(
                    &self.owner_window(),
                    Some(self.upcast::<Element>()),
                    "text/css".into(),
                    None, // todo handle location
                    None, // todo handle title
                    sheet,
                    None, // constructor_document
                    CanGc::note(),
                )
            })
        })
    }

    fn create_owned_contents_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        let cache_key = self.stylesheetcontents_cache_key.borrow_mut().take()?;
        if cache_key.is_uniquely_owned() {
            StylesheetContentsCache::remove(cache_key);
            return None;
        }

        let stylesheet_with_shared_contents = self.stylesheet.borrow().clone()?;
        let lock = stylesheet_with_shared_contents.shared_lock.clone();
        let guard = stylesheet_with_shared_contents.shared_lock.read();
        let stylesheet_with_owned_contents = Arc::new(Stylesheet {
            contents: lock.wrap(
                stylesheet_with_shared_contents
                    .contents(&guard)
                    .deep_clone(&lock, None, &guard),
            ),
            shared_lock: lock,
            media: stylesheet_with_shared_contents.media.clone(),
            disabled: AtomicBool::new(
                stylesheet_with_shared_contents
                    .disabled
                    .load(Ordering::SeqCst),
            ),
        });

        Some(stylesheet_with_owned_contents)
    }

    fn clean_stylesheet_ownership(&self) {
        if let Some(cssom_stylesheet) = self.cssom_stylesheet.get() {
            // If the CSSOMs change from having an owner node to being ownerless, they may still
            // potentially modify shared stylesheets. Thus, create an new `Stylesheet` with owned
            // `StylesheetContents` to ensure that the potentially modifications are only made on
            // the owned `StylesheetContents`.
            if let Some(stylesheet) = self.create_owned_contents_stylesheet() {
                let guard = stylesheet.shared_lock.read();
                cssom_stylesheet.update_style_stylesheet(&stylesheet, &guard);
            }
            cssom_stylesheet.set_owner_node(None);
        }
        self.cssom_stylesheet.set(None);
    }

    fn remove_stylesheet(&self) {
        self.clean_stylesheet_ownership();
        if let Some(s) = self.stylesheet.borrow_mut().take() {
            self.stylesheet_list_owner()
                .remove_stylesheet(StylesheetSource::Element(Dom::from_ref(self.upcast())), &s);
            let _ = self.stylesheetcontents_cache_key.borrow_mut().take();
        }
    }
}

impl VirtualMethods for HTMLStyleElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .children_changed(mutation, can_gc);

        // https://html.spec.whatwg.org/multipage/#update-a-style-block
        // Handles the case when:
        // "The element is not on the stack of open elements of an HTML parser or XML parser,
        // and one of its child nodes is modified by a script."
        // TODO: Handle Text child contents being mutated.
        let node = self.upcast::<Node>();
        if (node.is_in_a_document_tree() || node.is_in_a_shadow_tree()) &&
            !self.in_stack_of_open_elements.get()
        {
            self.parse_own_css();
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);

        // https://html.spec.whatwg.org/multipage/#update-a-style-block
        // Handles the case when:
        // "The element is not on the stack of open elements of an HTML parser or XML parser,
        // and it becomes connected or disconnected."
        if context.tree_connected && !self.in_stack_of_open_elements.get() {
            self.parse_own_css();
        }
    }

    fn pop(&self) {
        self.super_type().unwrap().pop();

        // https://html.spec.whatwg.org/multipage/#update-a-style-block
        // Handles the case when:
        // "The element is popped off the stack of open elements of an HTML parser or XML parser."
        self.in_stack_of_open_elements.set(false);
        if self.upcast::<Node>().is_in_a_document_tree() {
            self.parse_own_css();
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context, can_gc);
        }

        if context.tree_connected {
            self.remove_stylesheet();
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation, can_gc);
        }

        let node = self.upcast::<Node>();
        if !(node.is_in_a_document_tree() || node.is_in_a_shadow_tree()) ||
            self.in_stack_of_open_elements.get()
        {
            return;
        }

        if attr.name() == "type" {
            if let AttributeMutation::Set(Some(old_value), _) = mutation {
                if **old_value == **attr.value() {
                    return;
                }
            }
            self.remove_stylesheet();
            self.parse_own_css();
        } else if attr.name() == "media" {
            if let Some(ref stylesheet) = *self.stylesheet.borrow_mut() {
                let shared_lock = node.owner_doc().style_shared_lock().clone();
                let mut guard = shared_lock.write();
                let media = stylesheet.media.write_with(&mut guard);
                match mutation {
                    AttributeMutation::Set(..) => *media = self.create_media_list(&attr.value()),
                    AttributeMutation::Removed => *media = StyleMediaList::empty(),
                };
                self.owner_document().invalidate_stylesheets();
            }
        }
    }
}

impl StylesheetOwner for HTMLStyleElement {
    fn increment_pending_loads_count(&self) {
        self.pending_loads.set(self.pending_loads.get() + 1)
    }

    fn load_finished(&self, succeeded: bool) -> Option<bool> {
        assert!(self.pending_loads.get() > 0, "What finished?");
        if !succeeded {
            self.any_failed_load.set(true);
        }

        self.pending_loads.set(self.pending_loads.get() - 1);
        if self.pending_loads.get() != 0 {
            return None;
        }

        let any_failed = self.any_failed_load.get();
        self.any_failed_load.set(false);
        Some(any_failed)
    }

    fn parser_inserted(&self) -> bool {
        self.parser_inserted.get()
    }

    fn referrer_policy(&self) -> ReferrerPolicy {
        ReferrerPolicy::EmptyString
    }

    fn set_origin_clean(&self, origin_clean: bool) {
        if let Some(stylesheet) = self.get_cssom_stylesheet() {
            stylesheet.set_origin_clean(origin_clean);
        }
    }
}

impl HTMLStyleElementMethods<crate::DomTypeHolder> for HTMLStyleElement {
    /// <https://drafts.csswg.org/cssom/#dom-linkstyle-sheet>
    fn GetSheet(&self) -> Option<DomRoot<DOMStyleSheet>> {
        self.get_cssom_stylesheet().map(DomRoot::upcast)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-style-disabled>
    fn Disabled(&self) -> bool {
        self.get_cssom_stylesheet()
            .is_some_and(|sheet| sheet.disabled())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-style-disabled>
    fn SetDisabled(&self, value: bool) {
        if let Some(sheet) = self.get_cssom_stylesheet() {
            sheet.set_disabled(value);
        }
    }

    // <https://html.spec.whatwg.org/multipage/#HTMLStyleElement-partial>
    make_getter!(Type, "type");

    // <https://html.spec.whatwg.org/multipage/#HTMLStyleElement-partial>
    make_setter!(SetType, "type");

    // <https://html.spec.whatwg.org/multipage/#attr-style-media>
    make_getter!(Media, "media");

    // <https://html.spec.whatwg.org/multipage/#attr-style-media>
    make_setter!(SetMedia, "media");
}
