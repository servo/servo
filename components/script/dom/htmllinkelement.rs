/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser as CssParser;
use document_loader::LoadType;
use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::js::{RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::{AttributeMutation, Element, ElementCreator};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use encoding::EncodingRef;
use encoding::all::UTF_8;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use layout_interface::{LayoutChan, Msg};
use msg::constellation_msg::ConstellationChan;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata};
use network_listener::{NetworkListener, PreInvoke};
use script_traits::{MozBrowserEvent, ScriptMsg as ConstellationMsg};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::default::Default;
use std::mem;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use style::media_queries::{MediaQueryList, parse_media_query_list};
use style::servo::Stylesheet;
use style::stylesheets::Origin;
use url::Url;
use util::str::{DOMString, HTML_SPACE_CHARACTERS};

no_jsmanaged_fields!(Stylesheet);

#[dom_struct]
pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableHeap<JS<DOMTokenList>>,
    stylesheet: DOMRefCell<Option<Arc<Stylesheet>>>,

    /// https://html.spec.whatwg.org/multipage/#a-style-sheet-that-is-blocking-scripts
    parser_inserted: Cell<bool>,
}

impl HTMLLinkElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document,
                     creator: ElementCreator) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            rel_list: Default::default(),
            parser_inserted: Cell::new(creator == ElementCreator::ParserCreated),
            stylesheet: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document,
               creator: ElementCreator) -> Root<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, prefix, document, creator);
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }
}

fn get_attr(element: &Element, local_name: &Atom) -> Option<String> {
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

        let rel = get_attr(self.upcast(), &atom!("rel"));
        match attr.local_name() {
            &atom!("href") => {
                if string_is_stylesheet(&rel) {
                    self.handle_stylesheet_url(&attr.value());
                } else if is_favicon(&rel) {
                    let sizes = get_attr(self.upcast(), &atom!("sizes"));
                    self.handle_favicon_url(rel.as_ref().unwrap(), &attr.value(), &sizes);
                }
            },
            &atom!("sizes") => {
                if is_favicon(&rel) {
                    if let Some(ref href) = get_attr(self.upcast(), &atom!("href")) {
                        self.handle_favicon_url(rel.as_ref().unwrap(), href, &Some(attr.value().to_string()));
                    }
                }
            },
            &atom!("media") => {
                if string_is_stylesheet(&rel) {
                    self.handle_stylesheet_url(&attr.value());
                }
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("rel") => AttrValue::from_serialized_tokenlist(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            let element = self.upcast();

            let rel = get_attr(element, &atom!("rel"));
            let href = get_attr(element, &atom!("href"));
            let sizes = get_attr(self.upcast(), &atom!("sizes"));

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
    fn handle_stylesheet_url(&self, href: &str) {
        let document = document_from_node(self);
        match document.base_url().join(href) {
            Ok(url) => {
                let element = self.upcast::<Element>();

                let mq_attribute = element.get_attribute(&ns!(), &atom!("media"));
                let value = mq_attribute.r().map(|a| a.value());
                let mq_str = match value {
                    Some(ref value) => &***value,
                    None => "",
                };
                let mut css_parser = CssParser::new(&mq_str);
                let media = parse_media_query_list(&mut css_parser);

                // TODO: #8085 - Don't load external stylesheets if the node's mq doesn't match.
                let script_chan = document.window().networking_task_source();
                let elem = Trusted::new(self, script_chan.clone());

                let context = Arc::new(Mutex::new(StylesheetContext {
                    elem: elem,
                    media: Some(media),
                    data: vec!(),
                    metadata: None,
                    url: url.clone(),
                }));

                let (action_sender, action_receiver) = ipc::channel().unwrap();
                let listener = NetworkListener {
                    context: context,
                    script_chan: script_chan,
                };
                let response_target = AsyncResponseTarget {
                    sender: action_sender,
                };
                ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
                    listener.notify(message.to().unwrap());
                });

                if self.parser_inserted.get() {
                    document.increment_script_blocking_stylesheet_count();
                }
                document.load_async(LoadType::Stylesheet(url), response_target);
            }
            Err(e) => debug!("Parsing url {} failed: {}", href, e)
        }
    }

    fn handle_favicon_url(&self, rel: &str, href: &str, sizes: &Option<String>) {
        let document = document_from_node(self);
        match document.base_url().join(href) {
            Ok(url) => {
                let ConstellationChan(ref chan) = document.window().constellation_chan();
                let event = ConstellationMsg::NewFavicon(url.clone());
                chan.send(event).unwrap();

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

/// The context required for asynchronously loading an external stylesheet.
struct StylesheetContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLLinkElement>,
    media: Option<MediaQueryList>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The initial URL requested.
    url: Url,
}

impl PreInvoke for StylesheetContext {}

impl AsyncResponseListener for StylesheetContext {
    fn headers_available(&mut self, metadata: Metadata) {
        self.metadata = Some(metadata);
    }

    fn data_available(&mut self, payload: Vec<u8>) {
        let mut payload = payload;
        self.data.append(&mut payload);
    }

    fn response_complete(&mut self, _status: Result<(), String>) {
        let data = mem::replace(&mut self.data, vec!());
        let metadata = self.metadata.take().unwrap();
        // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
        let environment_encoding = UTF_8 as EncodingRef;
        let protocol_encoding_label = metadata.charset.as_ref().map(|s| &**s);
        let final_url = metadata.final_url;

        let elem = self.elem.root();
        let win = window_from_node(&*elem);

        let mut sheet = Stylesheet::from_bytes(&data, final_url, protocol_encoding_label,
                                               Some(environment_encoding), Origin::Author,
                                               win.css_error_reporter());
        let media = self.media.take().unwrap();
        sheet.set_media(Some(media));
        let sheet = Arc::new(sheet);

        let elem = elem.r();
        let document = document_from_node(elem);
        let document = document.r();

        let win = window_from_node(elem);
        let LayoutChan(ref layout_chan) = win.r().layout_chan();
        layout_chan.send(Msg::AddStylesheet(sheet.clone())).unwrap();

        *elem.stylesheet.borrow_mut() = Some(sheet);
        document.invalidate_stylesheets();
        if elem.parser_inserted.get() {
            document.decrement_script_blocking_stylesheet_count();
        }
        document.finish_load(LoadType::Stylesheet(self.url.clone()));
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
    make_setter!(SetRel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-link-media
    make_getter!(Media, "media");

    // https://html.spec.whatwg.org/multipage/#dom-link-media
    make_setter!(SetMedia, "media");

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
        self.rel_list.or_init(|| DOMTokenList::new(self.upcast(), &atom!("rel")))
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
}
