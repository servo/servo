/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use num_traits::ToPrimitive;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use style::attr::AttrValue;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{reflect_referrer_policy_attribute, AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{BindContext, Node, NodeTraits};
use crate::dom::urlhelper::UrlHelper;
use crate::dom::virtualmethods::VirtualMethods;
use crate::links::{follow_hyperlink, LinkRelations};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableDom<DOMTokenList>,

    #[no_trace]
    relations: Cell<LinkRelations>,
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
            relations: Cell::new(LinkRelations::empty()),
            url: DomRefCell::new(None),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLAnchorElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLAnchorElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-hyperlink-url-set>
    fn set_url(&self) {
        // Step 1. Set this element's url to null.
        *self.url.borrow_mut() = None;

        // Step 2. If this element's href content attribute is absent, then return.
        let attribute = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("href"));

        let Some(attribute) = attribute else {
            return;
        };

        // Step 3. Let url be the result of encoding-parsing a URL given this element's
        // href content attribute's value, relative to this element's node document.
        let document = self.owner_document();
        let url = document.encoding_parse_a_url(&attribute.value());

        // Step 4. If url is not failure, then set this element's url to url.
        if let Ok(url) = url {
            *self.url.borrow_mut() = Some(url);
        }
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
    fn update_href(&self, url: DOMString, can_gc: CanGc) {
        self.upcast::<Element>()
            .set_string_attribute(&local_name!("href"), url, can_gc);
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

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match *attr.local_name() {
            local_name!("rel") | local_name!("rev") => {
                self.relations
                    .set(LinkRelations::for_element(self.upcast()));
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        self.relations
            .set(LinkRelations::for_element(self.upcast()));
    }
}

impl HTMLAnchorElementMethods<crate::DomTypeHolder> for HTMLAnchorElement {
    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn SetText(&self, value: DOMString, can_gc: CanGc) {
        self.upcast::<Node>().SetTextContent(Some(value), can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    make_getter!(Rel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    fn SetRel(&self, rel: DOMString, can_gc: CanGc) {
        self.upcast::<Element>()
            .set_tokenlist_attribute(&local_name!("rel"), rel, can_gc);
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
    fn SetHash(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetHost(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetHostname(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetHref(&self, value: USVString, can_gc: CanGc) {
        self.upcast::<Element>().set_string_attribute(
            &local_name!("href"),
            DOMString::from_string(value.0),
            can_gc,
        );
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
    fn SetPassword(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetPathname(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetPort(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetProtocol(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetSearch(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
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
    fn SetUsername(&self, value: USVString, can_gc: CanGc) {
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
        self.update_href(url, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-referrerpolicy
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");
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
    fn activation_behavior(&self, event: &Event, target: &EventTarget, _can_gc: CanGc) {
        let element = self.as_element();
        let mouse_event = event.downcast::<MouseEvent>().unwrap();
        let mut ismap_suffix = None;

        // Step 1: If the target of the click event is an img element with an ismap attribute
        // specified, then server-side image map processing must be performed.
        if let Some(element) = target.downcast::<Element>() {
            if target.is::<HTMLImageElement>() && element.has_attribute(&local_name!("ismap")) {
                let target_node = element.upcast::<Node>();
                let rect = target_node.bounding_content_box_or_zero(CanGc::note());
                ismap_suffix = Some(format!(
                    "?{},{}",
                    mouse_event.ClientX().to_f32().unwrap() - rect.origin.x.to_f32_px(),
                    mouse_event.ClientY().to_f32().unwrap() - rect.origin.y.to_f32_px()
                ))
            }
        }

        // Step 2.
        //TODO: Download the link is `download` attribute is set.
        follow_hyperlink(element, self.relations.get(), ismap_suffix);
    }
}
