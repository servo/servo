/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::str::{DOMString, USVString};
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::Element;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, document_from_node};
use dom::urlhelper::UrlHelper;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use net_traits::ReferrerPolicy;
use num_traits::ToPrimitive;
use script_traits::MozBrowserEvent;
use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use std::default::Default;
use style::attr::AttrValue;

#[dom_struct]
pub struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableJS<DOMTokenList>,
    url: DOMRefCell<Option<ServoUrl>>,
}

impl HTMLAnchorElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
            url: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLAnchorElement> {
        Node::reflect_node(box HTMLAnchorElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLAnchorElementBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#concept-hyperlink-url-set
    fn set_url(&self) {
        let attribute = self.upcast::<Element>().get_attribute(&ns!(), &local_name!("href"));
        *self.url.borrow_mut() = attribute.and_then(|attribute| {
            let document = document_from_node(self);
            document.base_url().join(&attribute.value()).ok()
        });
    }

    // https://html.spec.whatwg.org/multipage/#reinitialise-url
    fn reinitialize_url(&self) {
        // Step 1.
        match *self.url.borrow() {
            Some(ref url) if url.scheme() == "blob" && url.cannot_be_a_base() => return,
            _ => (),
        }

        // Step 2.
        self.set_url();
    }

    // https://html.spec.whatwg.org/multipage/#update-href
    fn update_href(&self, url: DOMString) {
        self.upcast::<Element>().set_string_attribute(&local_name!("href"), url);
    }
}

impl VirtualMethods for HTMLAnchorElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl HTMLAnchorElementMethods for HTMLAnchorElement {
    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    make_getter!(Rel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    fn SetRel(&self, rel: DOMString) {
        self.upcast::<Element>().set_tokenlist_attribute(&local_name!("rel"), rel);
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rellist
    fn RelList(&self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| DOMTokenList::new(self.upcast(), &local_name!("rel")))
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_getter!(Coords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_setter!(SetCoords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-rev
    make_getter!(Rev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-a-rev
    make_setter!(SetRev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-a-shape
    make_getter!(Shape, "shape");

    // https://html.spec.whatwg.org/multipage/#dom-a-shape
    make_setter!(SetShape, "shape");

    // https://html.spec.whatwg.org/multipage/#attr-hyperlink-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#attr-hyperlink-target
    make_setter!(SetTarget, "target");

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash
    fn Hash(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 3.
            None => USVString(String::new()),
            Some(ref url) => {
                // Steps 3-4.
                UrlHelper::Hash(url)
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash
    fn SetHash(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            Some(ref url) if url.scheme() == "javascript" => return,
            None => return,
            // Steps 4-5.
            Some(url) => {
                UrlHelper::SetHash(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 6.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-host
    fn Host(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 3.
            None => USVString(String::new()),
            Some(ref url) => {
                if url.host().is_none() {
                    USVString(String::new())
                } else {
                    // Steps 4-5.
                    UrlHelper::Host(url)
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-host
    fn SetHost(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            Some(ref url) if url.cannot_be_a_base() => return,
            None => return,
            // Step 4.
            Some(url) => {
                UrlHelper::SetHost(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 5.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname
    fn Hostname(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 3.
            None => USVString(String::new()),
            Some(ref url) => {
                // Step 4.
                UrlHelper::Hostname(url)
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname
    fn SetHostname(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            Some(ref url) if url.cannot_be_a_base() => return,
            None => return,
            // Step 4.
            Some(url) => {
                UrlHelper::SetHostname(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 5.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    fn Href(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        USVString(match *self.url.borrow() {
            None => {
                match self.upcast::<Element>().get_attribute(&ns!(), &local_name!("href")) {
                    // Step 3.
                    None => String::new(),
                    // Step 4.
                    Some(attribute) => (**attribute.value()).to_owned(),
                }
            },
            // Step 5.
            Some(ref url) => url.as_str().to_owned(),
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    fn SetHref(&self, value: USVString) {
        self.upcast::<Element>().set_string_attribute(&local_name!("href"),
                                                      DOMString::from_string(value.0));
        self.set_url();
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-origin
    fn Origin(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        USVString(match *self.url.borrow() {
            None => {
                // Step 2.
                "".to_owned()
            },
            Some(ref url) => {
                // Step 3.
                url.origin().unicode_serialization()
            },
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-password
    fn Password(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 3.
            None => USVString(String::new()),
            // Steps 3-4.
            Some(ref url) => UrlHelper::Password(url)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-password
    fn SetPassword(&self, value: USVString) {
         // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            Some(ref url) if url.host().is_none() || url.cannot_be_a_base() => return,
            None => return,
            // Step 4.
            Some(url) => {
                UrlHelper::SetPassword(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 5.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname
    fn Pathname(&self) -> USVString {
          // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 3.
            None => USVString(String::new()),
            // Steps 4-5.
            Some(ref url) => UrlHelper::Pathname(url)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname
    fn SetPathname(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            Some(ref url) if url.cannot_be_a_base() => return,
            None => return,
            // Step 5.
            Some(url) => {
                UrlHelper::SetPathname(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 6.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-port
    fn Port(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 3.
            None => USVString(String::new()),
            // Step 4.
            Some(ref url) => UrlHelper::Port(url)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-port
    fn SetPort(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 3.
        let url = match self.url.borrow_mut().as_mut() {
            Some(ref url) if url.host().is_none() ||
                url.cannot_be_a_base() ||
                url.scheme() == "file" => return,
            None => return,
            // Step 4.
            Some(url) => {
                UrlHelper::SetPort(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 5.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol
    fn Protocol(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 2.
            None => USVString(":".to_owned()),
            // Step 3.
            Some(ref url) => UrlHelper::Protocol(url)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol
    fn SetProtocol(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        let url = match self.url.borrow_mut().as_mut() {
            // Step 2.
            None => return,
            // Step 3.
            Some(url) => {
                UrlHelper::SetProtocol(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 4.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-search
    fn Search(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 2.
            None => USVString(String::new()),
            // Step 3.
            Some(ref url) => UrlHelper::Search(url)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-search
    fn SetSearch(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            None => return,
            // Steps 4-5.
            // TODO add this element's node document character encoding as
            // encoding override (as described in the spec)
            Some(url) => {
                UrlHelper::SetSearch(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 6.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-username
    fn Username(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        match *self.url.borrow() {
            // Step 2.
            None => USVString(String::new()),
            // Step 3.
            Some(ref url) => UrlHelper::Username(url)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-username
    fn SetUsername(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 2.
        let url = match self.url.borrow_mut().as_mut() {
            // Step 3.
            Some(ref url) if url.host().is_none() || url.cannot_be_a_base() => return,
            None => return,
            // Step 4.
            Some(url) => {
                UrlHelper::SetUsername(url, value);
                DOMString::from(url.as_str())
            }
        };
        // Step 5.
        self.update_href(url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    fn Stringifier(&self) -> DOMString {
        DOMString::from(self.Href().0)
    }
}

impl Activatable for HTMLAnchorElement {
    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_instance_activatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#hyperlink
        // "a [...] element[s] with an href attribute [...] must [..] create a
        // hyperlink"
        // https://html.spec.whatwg.org/multipage/#the-a-element
        // "The activation behaviour of a elements *that create hyperlinks*"
        self.upcast::<Element>().has_attribute(&local_name!("href"))
    }


    //TODO:https://html.spec.whatwg.org/multipage/#the-a-element
    fn pre_click_activation(&self) {
    }

    //TODO:https://html.spec.whatwg.org/multipage/#the-a-element
    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self) {
    }

    //https://html.spec.whatwg.org/multipage/#the-a-element:activation-behaviour
    fn activation_behavior(&self, event: &Event, target: &EventTarget) {
        //Step 1. If the node document is not fully active, abort.
        let doc = document_from_node(self);
        if !doc.is_fully_active() {
            return;
        }
        //TODO: Step 2. Check if browsing context is specified and act accordingly.
        //Step 3. Handle <img ismap/>.
        let element = self.upcast::<Element>();
        let mouse_event = event.downcast::<MouseEvent>().unwrap();
        let mut ismap_suffix = None;
        if let Some(element) = target.downcast::<Element>() {
            if target.is::<HTMLImageElement>() && element.has_attribute(&local_name!("ismap")) {
                let target_node = element.upcast::<Node>();
                let rect = target_node.bounding_content_box_or_zero();
                ismap_suffix = Some(
                    format!("?{},{}", mouse_event.ClientX().to_f32().unwrap() - rect.origin.x.to_f32_px(),
                                      mouse_event.ClientY().to_f32().unwrap() - rect.origin.y.to_f32_px())
                )
            }
        }

        // Step 4.
        //TODO: Download the link is `download` attribute is set.

        // https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-delivery
        let referrer_policy = match self.RelList().Contains("noreferrer".into()) {
            true => Some(ReferrerPolicy::NoReferrer),
            false => None,
        };

        follow_hyperlink(element, ismap_suffix, referrer_policy);
    }

    //TODO:https://html.spec.whatwg.org/multipage/#the-a-element
    fn implicit_submission(&self, _ctrl_key: bool, _shift_key: bool, _alt_key: bool, _meta_key: bool) {
    }
}

/// https://html.spec.whatwg.org/multipage/#the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
fn is_current_browsing_context(target: DOMString) -> bool {
    target.is_empty() || target == "_self"
}

/// https://html.spec.whatwg.org/multipage/#following-hyperlinks-2
pub fn follow_hyperlink(subject: &Element, hyperlink_suffix: Option<String>, referrer_policy: Option<ReferrerPolicy>) {
    // Step 1: replace.
    // Step 2: source browsing context.
    // Step 3: target browsing context.
    let target = subject.get_attribute(&ns!(), &local_name!("target"));

    // Step 4: disown target's opener if needed.
    let attribute = subject.get_attribute(&ns!(), &local_name!("href")).unwrap();
    let mut href = attribute.Value();

    // Step 7: append a hyperlink suffix.
    // https://www.w3.org/Bugs/Public/show_bug.cgi?id=28925
    if let Some(suffix) = hyperlink_suffix {
        href.push_str(&suffix);
    }

    // Step 5: parse the URL.
    // Step 6: navigate to an error document if parsing failed.
    let document = document_from_node(subject);
    let url = match document.url().join(&href) {
        Ok(url) => url,
        Err(_) => return,
    };

    // Step 8: navigate to the URL.
    if let Some(target) = target {
        if PREFS.is_mozbrowser_enabled() && !is_current_browsing_context(target.Value()) {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowseropenwindow
            // TODO: referrer and opener
            // TODO: should we send the normalized url or the non-normalized href?
            let event = MozBrowserEvent::OpenWindow(url.into_string(), Some(String::from(target.Value())), None);
            document.trigger_mozbrowser_event(event);
            return
        }
    }

    debug!("following hyperlink to {}", url);

    let window = document.window();
    window.load_url(url, false, false, referrer_policy);
}
