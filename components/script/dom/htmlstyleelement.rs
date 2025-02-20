/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use cssparser::{Parser as CssParser, ParserInput};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use servo_arc::Arc;
use style::media_queries::MediaList;
use style::parser::ParserContext as CssParserContext;
use style::stylesheets::{AllowImportRules, CssRuleType, Origin, Stylesheet, UrlExtraData};
use style_traits::ParsingMode;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLStyleElementBinding::HTMLStyleElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, ElementCreator};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, ChildrenMutation, Node, NodeTraits, UnbindContext};
use crate::dom::stylesheet::StyleSheet as DOMStyleSheet;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;
use crate::stylesheet_loader::{StylesheetLoader, StylesheetOwner};

#[dom_struct]
pub(crate) struct HTMLStyleElement {
    htmlelement: HTMLElement,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    stylesheet: DomRefCell<Option<Arc<Stylesheet>>>,
    cssom_stylesheet: MutNullableDom<CSSStyleSheet>,
    /// <https://html.spec.whatwg.org/multipage/#a-style-sheet-that-is-blocking-scripts>
    parser_inserted: Cell<bool>,
    in_stack_of_open_elements: Cell<bool>,
    pending_loads: Cell<u32>,
    any_failed_load: Cell<bool>,
    line_number: u64,
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
            cssom_stylesheet: MutNullableDom::new(None),
            parser_inserted: Cell::new(creator.is_parser_created()),
            in_stack_of_open_elements: Cell::new(creator.is_parser_created()),
            pending_loads: Cell::new(0),
            any_failed_load: Cell::new(false),
            line_number: creator.return_line_number(),
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

    fn create_media_list(&self, mq_str: &str) -> MediaList {
        if mq_str.is_empty() {
            return MediaList::empty();
        }

        let window = self.owner_window();
        let doc = self.owner_document();
        let url_data = UrlExtraData(window.get_url().get_arc());
        let context = CssParserContext::new(
            Origin::Author,
            &url_data,
            Some(CssRuleType::Media),
            ParsingMode::DEFAULT,
            doc.quirks_mode(),
            /* namespaces = */ Default::default(),
            window.css_error_reporter(),
            None,
        );
        let mut input = ParserInput::new(mq_str);
        MediaList::parse(&context, &mut CssParser::new(&mut input))
    }

    pub(crate) fn parse_own_css(&self) {
        let node = self.upcast::<Node>();
        assert!(node.is_connected());

        // Step 4. of <https://html.spec.whatwg.org/multipage/#the-style-element%3Aupdate-a-style-block>
        let mut type_attribute = self.Type();
        type_attribute.make_ascii_lowercase();
        if !type_attribute.is_empty() && type_attribute != "text/css" {
            return;
        }

        let window = node.owner_window();
        let doc = self.owner_document();
        let data = node
            .GetTextContent()
            .expect("Element.textContent must be a string");
        let shared_lock = node.owner_doc().style_shared_lock().clone();
        let mq = Arc::new(shared_lock.wrap(self.create_media_list(&self.Media())));
        let loader = StylesheetLoader::for_element(self.upcast());
        let sheet = Stylesheet::from_str(
            &data,
            UrlExtraData(window.get_url().get_arc()),
            Origin::Author,
            mq,
            shared_lock,
            Some(&loader),
            window.css_error_reporter(),
            doc.quirks_mode(),
            AllowImportRules::Yes,
        );

        let sheet = Arc::new(sheet);

        // No subresource loads were triggered, queue load event
        if self.pending_loads.get() == 0 {
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue_simple_event(self.upcast(), atom!("load"));
        }

        self.set_stylesheet(sheet);
    }

    // FIXME(emilio): This is duplicated with HTMLLinkElement::set_stylesheet.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_stylesheet(&self, s: Arc<Stylesheet>) {
        let stylesheets_owner = self.stylesheet_list_owner();
        if let Some(ref s) = *self.stylesheet.borrow() {
            stylesheets_owner.remove_stylesheet(self.upcast(), s)
        }
        *self.stylesheet.borrow_mut() = Some(s.clone());
        self.clean_stylesheet_ownership();
        stylesheets_owner.add_stylesheet(self.upcast(), s);
    }

    pub(crate) fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub(crate) fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(
                    &self.owner_window(),
                    self.upcast::<Element>(),
                    "text/css".into(),
                    None, // todo handle location
                    None, // todo handle title
                    sheet,
                    CanGc::note(),
                )
            })
        })
    }

    fn clean_stylesheet_ownership(&self) {
        if let Some(cssom_stylesheet) = self.cssom_stylesheet.get() {
            cssom_stylesheet.set_owner(None);
        }
        self.cssom_stylesheet.set(None);
    }

    fn remove_stylesheet(&self) {
        if let Some(s) = self.stylesheet.borrow_mut().take() {
            self.clean_stylesheet_ownership();
            self.stylesheet_list_owner()
                .remove_stylesheet(self.upcast(), &s)
        }
    }
}

impl VirtualMethods for HTMLStyleElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        self.super_type().unwrap().children_changed(mutation);

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

    fn bind_to_tree(&self, context: &BindContext) {
        self.super_type().unwrap().bind_to_tree(context);

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

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_connected {
            self.remove_stylesheet();
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }

        let node = self.upcast::<Node>();
        if !(node.is_in_a_document_tree() || node.is_in_a_shadow_tree()) ||
            self.in_stack_of_open_elements.get()
        {
            return;
        }

        if attr.name() == "type" {
            if let AttributeMutation::Set(Some(old_value)) = mutation {
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
                    AttributeMutation::Set(_) => *media = self.create_media_list(&attr.value()),
                    AttributeMutation::Removed => *media = MediaList::empty(),
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
