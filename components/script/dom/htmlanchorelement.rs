/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use net_traits::request::Referrer;
use num_traits::ToPrimitive;
use script_traits::{HistoryEntryReplacement, LoadData, LoadOrigin};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use style::attr::AttrValue;

use crate::dom::activation::Activatable;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{referrer_policy_for_element, Element};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{document_from_node, Node};
use crate::dom::urlhelper::UrlHelper;
use crate::dom::virtualmethods::VirtualMethods;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableDom<DOMTokenList>,
    #[no_trace]
    url: DomRefCell<Option<ServoUrl>>,
}

impl HTMLAnchorElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
            url: DomRefCell::new(None),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLAnchorElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLAnchorElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    // https://html.spec.whatwg.org/multipage/#concept-hyperlink-url-set
    fn set_url(&self) {
        let attribute = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("href"));
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
        self.upcast::<Element>()
            .set_string_attribute(&local_name!("href"), url);
    }
}

impl VirtualMethods for HTMLAnchorElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
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
        self.upcast::<Element>()
            .set_tokenlist_attribute(&local_name!("rel"), rel);
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rellist
    fn RelList(&self) -> DomRoot<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(
                self.upcast(),
                &local_name!("rel"),
                Some(vec![
                    Atom::from("noopener"),
                    Atom::from("noreferrer"),
                    Atom::from("opener"),
                ]),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_getter!(Coords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_setter!(SetCoords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_atomic_setter!(SetName, "name");

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
            },
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
            },
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
            },
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
            },
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
            },
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
            },
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
                match self
                    .upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("href"))
                {
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
        self.upcast::<Element>()
            .set_string_attribute(&local_name!("href"), DOMString::from_string(value.0));
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
                url.origin().ascii_serialization()
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
            Some(ref url) => UrlHelper::Password(url),
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
            },
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
            Some(ref url) => UrlHelper::Pathname(url),
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
            },
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
            Some(ref url) => UrlHelper::Port(url),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-port
    fn SetPort(&self, value: USVString) {
        // Step 1.
        self.reinitialize_url();

        // Step 3.
        let url = match self.url.borrow_mut().as_mut() {
            Some(ref url)
                if url.host().is_none() || url.cannot_be_a_base() || url.scheme() == "file" =>
            {
                return;
            },
            None => return,
            // Step 4.
            Some(url) => {
                UrlHelper::SetPort(url, value);
                DOMString::from(url.as_str())
            },
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
            Some(ref url) => UrlHelper::Protocol(url),
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
            },
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
            Some(ref url) => UrlHelper::Search(url),
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
            },
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
            Some(ref url) => UrlHelper::Username(url),
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
            },
        };
        // Step 5.
        self.update_href(url);
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
        self.as_element().has_attribute(&local_name!("href"))
    }

    //https://html.spec.whatwg.org/multipage/#the-a-element:activation-behaviour
    fn activation_behavior(&self, event: &Event, target: &EventTarget) {
        let element = self.as_element();
        let mouse_event = event.downcast::<MouseEvent>().unwrap();
        let mut ismap_suffix = None;

        // Step 1: If the target of the click event is an img element with an ismap attribute
        // specified, then server-side image map processing must be performed.
        if let Some(element) = target.downcast::<Element>() {
            if target.is::<HTMLImageElement>() && element.has_attribute(&local_name!("ismap")) {
                let target_node = element.upcast::<Node>();
                let rect = target_node.bounding_content_box_or_zero();
                ismap_suffix = Some(format!(
                    "?{},{}",
                    mouse_event.ClientX().to_f32().unwrap() - rect.origin.x.to_f32_px(),
                    mouse_event.ClientY().to_f32().unwrap() - rect.origin.y.to_f32_px()
                ))
            }
        }

        // Step 2.
        //TODO: Download the link is `download` attribute is set.
        follow_hyperlink(element, ismap_suffix);
    }
}

/// <https://html.spec.whatwg.org/multipage/#get-an-element's-target>
pub fn get_element_target(subject: &Element) -> Option<DOMString> {
    if !(subject.is::<HTMLAreaElement>() ||
        subject.is::<HTMLAnchorElement>() ||
        subject.is::<HTMLFormElement>())
    {
        return None;
    }
    if subject.has_attribute(&local_name!("target")) {
        return Some(subject.get_string_attribute(&local_name!("target")));
    }

    let doc = document_from_node(subject).base_element();
    match doc {
        Some(doc) => {
            let element = doc.upcast::<Element>();
            if element.has_attribute(&local_name!("target")) {
                Some(element.get_string_attribute(&local_name!("target")))
            } else {
                None
            }
        },
        None => None,
    }
}

/// <https://html.spec.whatwg.org/multipage/#get-an-element's-noopener>
pub fn get_element_noopener(subject: &Element, target_attribute_value: Option<DOMString>) -> bool {
    if !(subject.is::<HTMLAreaElement>() ||
        subject.is::<HTMLAnchorElement>() ||
        subject.is::<HTMLFormElement>())
    {
        return false;
    }
    let target_is_blank = target_attribute_value
        .as_ref()
        .map_or(false, |target| target.to_lowercase() == "_blank");
    let link_types = match subject.get_attribute(&ns!(), &local_name!("rel")) {
        Some(rel) => rel.Value(),
        None => return target_is_blank,
    };
    link_types.contains("noreferrer") ||
        link_types.contains("noopener") ||
        (!link_types.contains("opener") && target_is_blank)
}

/// <https://html.spec.whatwg.org/multipage/#following-hyperlinks-2>
pub fn follow_hyperlink(subject: &Element, hyperlink_suffix: Option<String>) {
    // Step 1.
    if subject.cannot_navigate() {
        return;
    }
    // Step 2, done in Step 7.

    let document = document_from_node(subject);
    let window = document.window();

    // Step 3: source browsing context.
    let source = document.browsing_context().unwrap();

    // Step 4-5: target attribute.
    let target_attribute_value =
        if subject.is::<HTMLAreaElement>() || subject.is::<HTMLAnchorElement>() {
            get_element_target(subject)
        } else {
            None
        };
    // Step 6.
    let noopener = get_element_noopener(subject, target_attribute_value.clone());

    // Step 7.
    let (maybe_chosen, replace) = match target_attribute_value {
        Some(name) => {
            let (maybe_chosen, new) = source.choose_browsing_context(name, noopener);
            let replace = if new {
                HistoryEntryReplacement::Enabled
            } else {
                HistoryEntryReplacement::Disabled
            };
            (maybe_chosen, replace)
        },
        None => (
            Some(window.window_proxy()),
            HistoryEntryReplacement::Disabled,
        ),
    };

    // Step 8.
    let chosen = match maybe_chosen {
        Some(proxy) => proxy,
        None => return,
    };

    if let Some(target_document) = chosen.document() {
        let target_window = target_document.window();
        // Step 9, dis-owning target's opener, if necessary
        // will have been done as part of Step 7 above
        // in choose_browsing_context/create_auxiliary_browsing_context.

        // Step 10, 11. TODO: if parsing the URL failed, navigate to error page.
        let attribute = subject.get_attribute(&ns!(), &local_name!("href")).unwrap();
        let mut href = attribute.Value();
        // Step 11: append a hyperlink suffix.
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=28925
        if let Some(suffix) = hyperlink_suffix {
            href.push_str(&suffix);
        }
        let url = match document.base_url().join(&href) {
            Ok(url) => url,
            Err(_) => return,
        };

        // Step 12.
        let referrer_policy = referrer_policy_for_element(subject);

        // Step 13
        let referrer = match subject.get_attribute(&ns!(), &local_name!("rel")) {
            Some(ref link_types) if link_types.Value().contains("noreferrer") => {
                Referrer::NoReferrer
            },
            _ => target_window.upcast::<GlobalScope>().get_referrer(),
        };

        // Step 14
        let pipeline_id = target_window.upcast::<GlobalScope>().pipeline_id();
        let secure = target_window.upcast::<GlobalScope>().is_secure_context();
        let load_data = LoadData::new(
            LoadOrigin::Script(document.origin().immutable().clone()),
            url,
            Some(pipeline_id),
            referrer,
            referrer_policy,
            Some(secure),
        );
        let target = Trusted::new(target_window);
        let task = task!(navigate_follow_hyperlink: move || {
            debug!("following hyperlink to {}", load_data.url);
            target.root().load_url(replace, false, load_data);
        });
        target_window
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task, target_window.upcast())
            .unwrap();
    };
}
