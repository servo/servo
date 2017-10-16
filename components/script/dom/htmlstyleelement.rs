/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser as CssParser, ParserInput};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::HTMLStyleElementBinding;
use dom::bindings::codegen::Bindings::HTMLStyleElementBinding::HTMLStyleElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::cssstylesheet::CSSStyleSheet;
use dom::document::Document;
use dom::element::{Element, ElementCreator};
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, Node, UnbindContext, document_from_node, window_from_node};
use dom::stylesheet::StyleSheet as DOMStyleSheet;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use net_traits::ReferrerPolicy;
use servo_arc::Arc;
use std::cell::Cell;
use style::media_queries::parse_media_query_list;
use style::parser::ParserContext as CssParserContext;
use style::stylesheets::{CssRuleType, Stylesheet, Origin};
use style_traits::PARSING_MODE_DEFAULT;
use stylesheet_loader::{StylesheetLoader, StylesheetOwner};

#[dom_struct]
pub struct HTMLStyleElement {
    htmlelement: HTMLElement,
    #[ignore_heap_size_of = "Arc"]
    stylesheet: DomRefCell<Option<Arc<Stylesheet>>>,
    cssom_stylesheet: MutNullableDom<CSSStyleSheet>,
    /// https://html.spec.whatwg.org/multipage/#a-style-sheet-that-is-blocking-scripts
    parser_inserted: Cell<bool>,
    in_stack_of_open_elements: Cell<bool>,
    pending_loads: Cell<u32>,
    any_failed_load: Cell<bool>,
    line_number: u64,
}

impl HTMLStyleElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document,
                     creator: ElementCreator) -> HTMLStyleElement {
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

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document,
               creator: ElementCreator) -> DomRoot<HTMLStyleElement> {
        Node::reflect_node(Box::new(HTMLStyleElement::new_inherited(local_name, prefix, document, creator)),
                           document,
                           HTMLStyleElementBinding::Wrap)
    }

    pub fn parse_own_css(&self) {
        let node = self.upcast::<Node>();
        let element = self.upcast::<Element>();
        assert!(node.is_in_doc());

        let window = window_from_node(node);
        let doc = document_from_node(self);

        let mq_attribute = element.get_attribute(&ns!(), &local_name!("media"));
        let mq_str = match mq_attribute {
            Some(a) => String::from(&**a.value()),
            None => String::new(),
        };

        let data = node.GetTextContent().expect("Element.textContent must be a string");
        let url = window.get_url();
        let context = CssParserContext::new_for_cssom(&url,
                                                      Some(CssRuleType::Media),
                                                      PARSING_MODE_DEFAULT,
                                                      doc.quirks_mode());
        let shared_lock = node.owner_doc().style_shared_lock().clone();
        let mut input = ParserInput::new(&mq_str);
        let css_error_reporter = window.css_error_reporter();
        let mq = Arc::new(shared_lock.wrap(parse_media_query_list(&context,
                                                                  &mut CssParser::new(&mut input),
                                                                  css_error_reporter)));
        let loader = StylesheetLoader::for_element(self.upcast());
        let sheet = Stylesheet::from_str(&data, window.get_url(),
                                         Origin::Author, mq,
                                         shared_lock, Some(&loader),
                                         css_error_reporter,
                                         doc.quirks_mode(),
                                         self.line_number as u32);

        let sheet = Arc::new(sheet);

        // No subresource loads were triggered, just fire the load event now.
        if self.pending_loads.get() == 0 {
            self.upcast::<EventTarget>().fire_event(atom!("load"));
        }

        self.set_stylesheet(sheet);
    }

    // FIXME(emilio): This is duplicated with HTMLLinkElement::set_stylesheet.
    pub fn set_stylesheet(&self, s: Arc<Stylesheet>) {
        let doc = document_from_node(self);
        if let Some(ref s) = *self.stylesheet.borrow() {
            doc.remove_stylesheet(self.upcast(), s)
        }
        *self.stylesheet.borrow_mut() = Some(s.clone());
        self.cssom_stylesheet.set(None);
        doc.add_stylesheet(self.upcast(), s);
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(&window_from_node(self),
                                   self.upcast::<Element>(),
                                   "text/css".into(),
                                   None, // todo handle location
                                   None, // todo handle title
                                   sheet)
            })
        })
    }
}

impl VirtualMethods for HTMLStyleElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
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

    fn bind_to_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().bind_to_tree(tree_in_doc);

        // https://html.spec.whatwg.org/multipage/#update-a-style-block
        // Handles the case when:
        // "The element is not on the stack of open elements of an HTML parser or XML parser,
        // and it becomes connected or disconnected."
        if tree_in_doc && !self.in_stack_of_open_elements.get() {
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
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_in_doc {
            if let Some(s) = self.stylesheet.borrow_mut().take() {
                document_from_node(self).remove_stylesheet(self.upcast(), &s)
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
    // https://drafts.csswg.org/cssom/#dom-linkstyle-sheet
    fn GetSheet(&self) -> Option<DomRoot<DOMStyleSheet>> {
        self.get_cssom_stylesheet().map(DomRoot::upcast)
    }
}
