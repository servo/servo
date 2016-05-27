/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrValue;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::str::DOMString;
use dom::document::{Document, determine_policy_for_token};
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use std::ascii::AsciiExt;
use std::sync::Arc;
use string_cache::Atom;
use style::servo::Stylesheet;
use style::stylesheets::{CSSRule, Origin};
use style::viewport::ViewportRule;
use util::str::HTML_SPACE_CHARACTERS;

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
    stylesheet: DOMRefCell<Option<Arc<Stylesheet>>>,
}

impl HTMLMetaElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            stylesheet: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLMetaElementBinding::Wrap)
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    fn process_attributes(&self) {
        let element = self.upcast::<Element>();
        if let Some(name) = element.get_attribute(&ns!(), &atom!("name")).r() {
            let name = name.value().to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "viewport" {
                self.apply_viewport();
            }

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    fn apply_viewport(&self) {
        if !::util::prefs::get_pref("layout.viewport.enabled").as_boolean().unwrap_or(false) {
            return;
        }
        let element = self.upcast::<Element>();
        if let Some(content) = element.get_attribute(&ns!(), &atom!("content")).r() {
            let content = content.value();
            if !content.is_empty() {
                if let Some(translated_rule) = ViewportRule::from_meta(&**content) {
                    *self.stylesheet.borrow_mut() = Some(Arc::new(Stylesheet {
                        rules: vec![CSSRule::Viewport(translated_rule)],
                        origin: Origin::Author,
                        media: None,
                        // Viewport constraints are always recomputed on resize; they don't need to
                        // force all styles to be recomputed.
                        dirty_on_viewport_size_change: false,
                    }));
                    let doc = document_from_node(self);
                    doc.invalidate_stylesheets();
                }
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#meta-referrer
    fn apply_referrer(&self) {
        /*todo - I think this chould only run if document's policy hasnt yet
        been set. unclear - see:
        https://html.spec.whatwg.org/multipage/#meta-referrer
        https://w3c.github.io/webappsec-referrer-policy/#set-referrer-policy
        */
        let doc = document_from_node(self);
        if let Some(head) = doc.GetHead() {
            if head.upcast::<Node>().is_parent_of(self.upcast::<Node>()) {
                let element = self.upcast::<Element>();
                if let Some(content) = element.get_attribute(&ns!(), &atom!("content")).r() {
                    let content = content.value();
                    let content_val = content.trim();
                    if !content_val.is_empty() {
                        doc.set_referrer_policy(determine_policy_for_token(content_val));
                    }
                }
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
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            self.process_attributes();
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
