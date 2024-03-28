/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use cssparser::{Parser as CssParser, ParserInput};
use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use servo_arc::Arc;
use style::media_queries::MediaList;
use style::parser::ParserContext as CssParserContext;
use style::stylesheets::{AllowImportRules, CssRuleType, Origin, Stylesheet, UrlExtraData};
use style_traits::ParsingMode;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLStyleElementBinding::HTMLStyleElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::element::{Element, ElementCreator};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{
    document_from_node, stylesheets_owner_from_node, window_from_node, BindContext,
    ChildrenMutation, Node, UnbindContext,
};
use crate::dom::stylesheet::StyleSheet as DOMStyleSheet;
use crate::dom::virtualmethods::VirtualMethods;
use crate::stylesheet_loader::{StylesheetLoader, StylesheetOwner};

#[dom_struct]
pub struct HTMLStyleElement {
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

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
    ) -> DomRoot<HTMLStyleElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLStyleElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
        )
    }

    pub fn parse_own_css(&self) {
        let node = self.upcast::<Node>();
        let element = self.upcast::<Element>();
        assert!(node.is_connected());

        let window = window_from_node(node);
        let doc = document_from_node(self);

        let mq_attribute = element.get_attribute(&ns!(), &local_name!("media"));
        let mq_str = match mq_attribute {
            Some(a) => String::from(&**a.value()),
            None => String::new(),
        };

        let data = node
            .GetTextContent()
            .expect("Element.textContent must be a string");
        let url_data = UrlExtraData(window.get_url().get_arc());
        let css_error_reporter = window.css_error_reporter();
        let context = CssParserContext::new(
            Origin::Author,
            &url_data,
            Some(CssRuleType::Media),
            ParsingMode::DEFAULT,
            doc.quirks_mode(),
            /* namespaces = */ Default::default(),
            css_error_reporter,
            None,
        );
        let shared_lock = node.owner_doc().style_shared_lock().clone();
        let mut input = ParserInput::new(&mq_str);
        let mq =
            Arc::new(shared_lock.wrap(MediaList::parse(&context, &mut CssParser::new(&mut input))));
        let loader = StylesheetLoader::for_element(self.upcast());
        let sheet = Stylesheet::from_str(
            &data,
            UrlExtraData(window.get_url().get_arc()),
            Origin::Author,
            mq,
            shared_lock,
            Some(&loader),
            css_error_reporter,
            doc.quirks_mode(),
            AllowImportRules::Yes,
        );

        let sheet = Arc::new(sheet);

        // No subresource loads were triggered, queue load event
        if self.pending_loads.get() == 0 {
            let window = window_from_node(self);
            window
                .task_manager()
                .dom_manipulation_task_source()
                .queue_simple_event(self.upcast(), atom!("load"), &window);
        }

        self.set_stylesheet(sheet);
    }

    // FIXME(emilio): This is duplicated with HTMLLinkElement::set_stylesheet.
    #[allow(crown::unrooted_must_root)]
    pub fn set_stylesheet(&self, s: Arc<Stylesheet>) {
        let stylesheets_owner = stylesheets_owner_from_node(self);
        if let Some(ref s) = *self.stylesheet.borrow() {
            stylesheets_owner.remove_stylesheet(self.upcast(), s)
        }
        *self.stylesheet.borrow_mut() = Some(s.clone());
        self.clean_stylesheet_ownership();
        stylesheets_owner.add_stylesheet(self.upcast(), s);
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(
                    &window_from_node(self),
                    self.upcast::<Element>(),
                    "text/css".into(),
                    None, // todo handle location
                    None, // todo handle title
                    sheet,
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
        if self.upcast::<Node>().is_in_doc() && !self.in_stack_of_open_elements.get() {
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
        if self.upcast::<Node>().is_in_doc() {
            self.parse_own_css();
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_connected {
            if let Some(s) = self.stylesheet.borrow_mut().take() {
                self.clean_stylesheet_ownership();
                stylesheets_owner_from_node(self).remove_stylesheet(self.upcast(), &s)
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

    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        None
    }

    fn set_origin_clean(&self, origin_clean: bool) {
        if let Some(stylesheet) = self.get_cssom_stylesheet() {
            stylesheet.set_origin_clean(origin_clean);
        }
    }
}

impl HTMLStyleElementMethods for HTMLStyleElement {
    /// <https://drafts.csswg.org/cssom/#dom-linkstyle-sheet>
    fn GetSheet(&self) -> Option<DomRoot<DOMStyleSheet>> {
        self.get_cssom_stylesheet().map(DomRoot::upcast)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-style-disabled>
    fn Disabled(&self) -> bool {
        self.get_cssom_stylesheet()
            .map_or(false, |sheet| sheet.disabled())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-style-disabled>
    fn SetDisabled(&self, value: bool) {
        if let Some(sheet) = self.get_cssom_stylesheet() {
            sheet.set_disabled(value);
        }
    }
}
