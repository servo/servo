/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser as CssParser;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root, RootedReference};
use dom::bindings::str::DOMString;
use dom::cssstylesheet::CSSStyleSheet;
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::{AttributeMutation, Element, ElementCreator};
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::stylesheet::StyleSheet as DOMStyleSheet;
use dom::virtualmethods::VirtualMethods;
use html5ever_atoms::LocalName;
use net_traits::ReferrerPolicy;
use script_traits::{MozBrowserEvent, ScriptMsg as ConstellationMsg};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::default::Default;
use std::sync::Arc;
use style::attr::AttrValue;
use style::media_queries::parse_media_query_list;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::Stylesheet;
use stylesheet_loader::{StylesheetLoader, StylesheetContextSource, StylesheetOwner};

unsafe_no_jsmanaged_fields!(Stylesheet);

#[dom_struct]
pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableJS<DOMTokenList>,
    #[ignore_heap_size_of = "Arc"]
    stylesheet: DOMRefCell<Option<Arc<Stylesheet>>>,
    cssom_stylesheet: MutNullableJS<CSSStyleSheet>,

    /// https://html.spec.whatwg.org/multipage/#a-style-sheet-that-is-blocking-scripts
    parser_inserted: Cell<bool>,
    /// The number of loads that this link element has triggered (could be more
    /// than one because of imports) and have not yet finished.
    pending_loads: Cell<u32>,
    /// Whether any of the loads have failed.
    any_failed_load: Cell<bool>,
}

impl HTMLLinkElement {
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document,
                     creator: ElementCreator) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
            parser_inserted: Cell::new(creator == ElementCreator::ParserCreated),
            stylesheet: DOMRefCell::new(None),
            cssom_stylesheet: MutNullableJS::new(None),
            pending_loads: Cell::new(0),
            any_failed_load: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document,
               creator: ElementCreator) -> Root<HTMLLinkElement> {
        Node::reflect_node(box HTMLLinkElement::new_inherited(local_name, prefix, document, creator),
                           document,
                           HTMLLinkElementBinding::Wrap)
    }

    pub fn set_stylesheet(&self, s: Arc<Stylesheet>) {
        assert!(self.stylesheet.borrow().is_none());
        *self.stylesheet.borrow_mut() = Some(s);
    }


    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub fn get_cssom_stylesheet(&self) -> Option<Root<CSSStyleSheet>> {
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

fn get_attr(element: &Element, local_name: &LocalName) -> Option<String> {
    let elem = element.get_attribute(&ns!(), local_name);
    elem.map(|e| {
        let value = e.value();
        (**value).to_owned()
    })
}

fn string_is_stylesheet(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => {
            let mut found_stylesheet = false;
            for s in value.split(HTML_SPACE_CHARACTERS).into_iter() {
                if s.eq_ignore_ascii_case("alternate") {
                    return false;
                }

                if s.eq_ignore_ascii_case("stylesheet") {
                    found_stylesheet = true;
                }
            }
            found_stylesheet
        },
        None => false,
    }
}

/// Favicon spec usage in accordance with CEF implementation:
/// only url of icon is required/used
/// https://html.spec.whatwg.org/multipage/#rel-icon
fn is_favicon(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => {
            value.split(HTML_SPACE_CHARACTERS)
                .any(|s| s.eq_ignore_ascii_case("icon") || s.eq_ignore_ascii_case("apple-touch-icon"))
        },
        None => false,
    }
}

impl VirtualMethods for HTMLLinkElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if !self.upcast::<Node>().is_in_doc() || mutation == AttributeMutation::Removed {
            return;
        }

        let rel = get_attr(self.upcast(), &local_name!("rel"));
        match attr.local_name() {
            &local_name!("href") => {
                if string_is_stylesheet(&rel) {
                    self.handle_stylesheet_url(&attr.value());
                } else if is_favicon(&rel) {
                    let sizes = get_attr(self.upcast(), &local_name!("sizes"));
                    self.handle_favicon_url(rel.as_ref().unwrap(), &attr.value(), &sizes);
                }
            },
            &local_name!("sizes") => {
                if is_favicon(&rel) {
                    if let Some(ref href) = get_attr(self.upcast(), &local_name!("href")) {
                        self.handle_favicon_url(rel.as_ref().unwrap(), href, &Some(attr.value().to_string()));
                    }
                }
            },
            &local_name!("media") => {
                if string_is_stylesheet(&rel) {
                    if let Some(href) = self.upcast::<Element>().get_attribute(&ns!(), &local_name!("href")) {
                        self.handle_stylesheet_url(&href.value());
                    }
                }
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            let element = self.upcast();

            let rel = get_attr(element, &local_name!("rel"));
            let href = get_attr(element, &local_name!("href"));
            let sizes = get_attr(self.upcast(), &local_name!("sizes"));

            match href {
                Some(ref href) if string_is_stylesheet(&rel) => {
                    self.handle_stylesheet_url(href);
                }
                Some(ref href) if is_favicon(&rel) => {
                    self.handle_favicon_url(rel.as_ref().unwrap(), href, &sizes);
                }
                _ => {}
            }
        }
    }
}


impl HTMLLinkElement {
    /// https://html.spec.whatwg.org/multipage/#concept-link-obtain
    fn handle_stylesheet_url(&self, href: &str) {
        let document = document_from_node(self);
        if document.browsing_context().is_none() {
            return;
        }

        // Step 1.
        if href.is_empty() {
            return;
        }

        // Step 2.
        let url = match document.base_url().join(href) {
            Ok(url) => url,
            Err(e) => {
                debug!("Parsing url {} failed: {}", href, e);
                return;
            }
        };

        let element = self.upcast::<Element>();

        let mq_attribute = element.get_attribute(&ns!(), &local_name!("media"));
        let value = mq_attribute.r().map(|a| a.value());
        let mq_str = match value {
            Some(ref value) => &***value,
            None => "",
        };

        let mut css_parser = CssParser::new(&mq_str);
        let media = parse_media_query_list(&mut css_parser);

        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.r().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        // TODO: #8085 - Don't load external stylesheets if the node's mq
        // doesn't match.
        let loader = StylesheetLoader::for_element(self.upcast());
        loader.load(StylesheetContextSource::LinkElement {
            url: url,
            media: Some(media),
        }, integrity_metadata.to_owned());
    }

    fn handle_favicon_url(&self, rel: &str, href: &str, sizes: &Option<String>) {
        let document = document_from_node(self);
        match document.base_url().join(href) {
            Ok(url) => {
                let event = ConstellationMsg::NewFavicon(url.clone());
                document.window().upcast::<GlobalScope>().constellation_chan().send(event).unwrap();

                let mozbrowser_event = match *sizes {
                    Some(ref sizes) => MozBrowserEvent::IconChange(rel.to_owned(), url.to_string(), sizes.to_owned()),
                    None => MozBrowserEvent::IconChange(rel.to_owned(), url.to_string(), "".to_owned())
                };
                document.trigger_mozbrowser_event(mozbrowser_event);
            }
            Err(e) => debug!("Parsing url {} failed: {}", href, e)
        }
    }
}

impl StylesheetOwner for HTMLLinkElement {
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
        if self.RelList().Contains("noreferrer".into()) {
            return Some(ReferrerPolicy::NoReferrer)
        }

        None
    }
}

impl HTMLLinkElementMethods for HTMLLinkElement {
    // https://html.spec.whatwg.org/multipage/#dom-link-href
    make_url_getter!(Href, "href");

    // https://html.spec.whatwg.org/multipage/#dom-link-href
    make_setter!(SetHref, "href");

    // https://html.spec.whatwg.org/multipage/#dom-link-rel
    make_getter!(Rel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-link-rel
    fn SetRel(&self, rel: DOMString) {
        self.upcast::<Element>().set_tokenlist_attribute(&local_name!("rel"), rel);
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-media
    make_getter!(Media, "media");

    // https://html.spec.whatwg.org/multipage/#dom-link-media
    make_setter!(SetMedia, "media");

    // https://html.spec.whatwg.org/multipage/#dom-link-integrity
    make_getter!(Integrity, "integrity");

    // https://html.spec.whatwg.org/multipage/#dom-link-integrity
    make_setter!(SetIntegrity, "integrity");

    // https://html.spec.whatwg.org/multipage/#dom-link-hreflang
    make_getter!(Hreflang, "hreflang");

    // https://html.spec.whatwg.org/multipage/#dom-link-hreflang
    make_setter!(SetHreflang, "hreflang");

    // https://html.spec.whatwg.org/multipage/#dom-link-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-link-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-link-rellist
    fn RelList(&self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| DOMTokenList::new(self.upcast(), &local_name!("rel")))
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-charset
    make_getter!(Charset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-link-charset
    make_setter!(SetCharset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-link-rev
    make_getter!(Rev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-link-rev
    make_setter!(SetRev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-link-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#dom-link-target
    make_setter!(SetTarget, "target");

    // https://drafts.csswg.org/cssom/#dom-linkstyle-sheet
    fn GetSheet(&self) -> Option<Root<DOMStyleSheet>> {
        self.get_cssom_stylesheet().map(Root::upcast)
    }
}
