/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::node::{
    document_from_node, stylesheets_owner_from_node, window_from_node, BindContext, Node,
    UnbindContext,
};
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use parking_lot::RwLock;
use servo_arc::Arc;
use servo_config::pref;
use std::sync::atomic::AtomicBool;
use style::media_queries::MediaList;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::{CssRule, CssRules, Origin, Stylesheet, StylesheetContents, ViewportRule};

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
    #[ignore_malloc_size_of = "Arc"]
    stylesheet: DomRefCell<Option<Arc<Stylesheet>>>,
    cssom_stylesheet: MutNullableDom<CSSStyleSheet>,
}

impl HTMLMetaElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            stylesheet: DomRefCell::new(None),
            cssom_stylesheet: MutNullableDom::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLMetaElement> {
        Node::reflect_node(
            Box::new(HTMLMetaElement::new_inherited(local_name, prefix, document)),
            document,
        )
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

    fn process_attributes(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "viewport" {
                self.apply_viewport();
            }

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    #[allow(unrooted_must_root)]
    fn apply_viewport(&self) {
        if !pref!(layout.viewport.enabled) {
            return;
        }
        let element = self.upcast::<Element>();
        if let Some(ref content) = element.get_attribute(&ns!(), &local_name!("content")) {
            let content = content.value();
            if !content.is_empty() {
                if let Some(translated_rule) = ViewportRule::from_meta(&**content) {
                    let stylesheets_owner = stylesheets_owner_from_node(self);
                    let document = document_from_node(self);
                    let shared_lock = document.style_shared_lock();
                    let rule = CssRule::Viewport(Arc::new(shared_lock.wrap(translated_rule)));
                    let sheet = Arc::new(Stylesheet {
                        contents: StylesheetContents {
                            rules: CssRules::new(vec![rule], shared_lock),
                            origin: Origin::Author,
                            namespaces: Default::default(),
                            quirks_mode: document.quirks_mode(),
                            url_data: RwLock::new(window_from_node(self).get_url()),
                            source_map_url: RwLock::new(None),
                            source_url: RwLock::new(None),
                        },
                        media: Arc::new(shared_lock.wrap(MediaList::empty())),
                        shared_lock: shared_lock.clone(),
                        disabled: AtomicBool::new(false),
                    });
                    *self.stylesheet.borrow_mut() = Some(sheet.clone());
                    stylesheets_owner.add_stylesheet(self.upcast(), sheet);
                }
            }
        }
    }

    fn process_referrer_attribute(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    fn apply_referrer(&self) {
        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            if let Some(head) = parent.downcast::<HTMLHeadElement>() {
                head.set_document_referrer();
            }
        }
    }
}

impl HTMLMetaElementMethods for HTMLMetaElement {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");
}

impl VirtualMethods for HTMLMetaElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if context.tree_connected {
            self.process_attributes();
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }

        self.process_referrer_attribute();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_connected {
            self.process_referrer_attribute();

            if let Some(s) = self.stylesheet.borrow_mut().take() {
                stylesheets_owner_from_node(self).remove_stylesheet(self.upcast(), &s);
            }
        }
    }
}
