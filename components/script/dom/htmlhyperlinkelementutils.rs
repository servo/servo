/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::{local_name, namespace_url, ns};
use servo_url::ServoUrl;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::element::Element;
use crate::dom::node::NodeTraits;
use crate::dom::urlhelper::UrlHelper;
use crate::script_runtime::CanGc;

pub(crate) trait HyperlinkElement {
    fn get_url(&self) -> &DomRefCell<Option<ServoUrl>>;
}

/// <https://html.spec.whatwg.org/multipage/#htmlhyperlinkelementutils>
pub(crate) trait HyperlinkElementTraits {
    fn get_hash(&self) -> USVString;
    fn set_hash(&self, value: USVString, can_gc: CanGc);
    fn get_host(&self) -> USVString;
    fn set_host(&self, value: USVString, can_gc: CanGc);
    fn get_hostname(&self) -> USVString;
    fn set_hostname(&self, value: USVString, can_gc: CanGc);
    fn get_href(&self) -> USVString;
    fn set_href(&self, value: USVString, can_gc: CanGc);
    fn get_origin(&self) -> USVString;
    fn get_password(&self) -> USVString;
    fn set_password(&self, value: USVString, can_gc: CanGc);
    fn get_pathname(&self) -> USVString;
    fn set_pathname(&self, value: USVString, can_gc: CanGc);
    fn get_port(&self) -> USVString;
    fn set_port(&self, value: USVString, can_gc: CanGc);
    fn get_protocol(&self) -> USVString;
    fn set_protocol(&self, value: USVString, can_gc: CanGc);
    fn get_search(&self) -> USVString;
    fn set_search(&self, value: USVString, can_gc: CanGc);
    fn get_username(&self) -> USVString;
    fn set_url(&self);
    fn set_username(&self, value: USVString, can_gc: CanGc);
    fn update_href(&self, url: DOMString, can_gc: CanGc);
    fn reinitialize_url(&self);
}

impl<T: HyperlinkElement + DerivedFrom<Element> + Castable + NodeTraits> HyperlinkElementTraits
    for T
{
    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash>
    fn get_hash(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url is null, or url's fragment is either null or the empty string, return
            // the empty string.
            None => USVString(String::new()),
            Some(ref url) if url.fragment().is_none() || url.fragment() == Some("") => {
                USVString(String::new())
            },
            Some(ref url) => {
                // Step 4. Return "#", followed by url's fragment.
                UrlHelper::Hash(url)
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash>
    fn set_hash(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null, then return.
            None => return,
            // Step 4. If the given value is the empty string, set url's fragment to null.
            // Note this step is taken care of by UrlHelper::SetHash when the value is Some
            // Steps 5. Otherwise:
            Some(url) => {
                // Step 5.1. Let input be the given value with a single leading "#" removed, if any.
                // Step 5.2. Set url's fragment to the empty string.
                // Note these steps are taken care of by UrlHelper::SetHash
                UrlHelper::SetHash(url, value);

                // Step 5.4.  Basic URL parse input, with url as url and fragment state as state
                // override.
                DOMString::from(url.as_str())
            },
        };

        // Step 6. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-host>
    fn get_host(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url or url's host is null, return the empty string.
            None => USVString(String::new()),
            Some(ref url) => {
                if url.host().is_none() {
                    USVString(String::new())
                } else {
                    // Step 4. If url's port is null, return url's host, serialized.
                    // Step 5. Return url's host, serialized, followed by ":" and url's port,
                    // serialized.
                    UrlHelper::Host(url)
                }
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-host>
    fn set_host(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url or url's host is null, return the empty string.
            Some(ref url) if url.cannot_be_a_base() => return,
            None => return,
            // Step 4. Basic URL parse the given value, with url as url and host state as state
            // override.
            Some(url) => {
                UrlHelper::SetHost(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 5. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname>
    fn get_hostname(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url or url's host is null, return the empty string.
            None => USVString(String::new()),
            Some(ref url) => {
                // Step 4. Return url's host, serialized.
                UrlHelper::Hostname(url)
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname>
    fn set_hostname(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null or url has an opaque path, then return.
            None => return,
            Some(ref url) if url.cannot_be_a_base() => return,
            // Step 4. Basic URL parse the given value, with url as url and hostname state as state
            // override.
            Some(url) => {
                UrlHelper::SetHostname(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 5. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-href>
    fn get_href(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        USVString(match *self.get_url().borrow() {
            None => {
                match self
                    .upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("href"))
                {
                    // Step 3. If url is null and this has no href content attribute, return the
                    // empty string.
                    None => String::new(),

                    // Step 4. Otherwise, if url is null, return this's href content attribute's value.
                    Some(attribute) => (**attribute.value()).to_owned(),
                }
            },
            // Step 5. Return url, serialized.
            Some(ref url) => url.as_str().to_owned(),
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    fn set_href(&self, value: USVString, can_gc: CanGc) {
        self.upcast::<Element>().set_string_attribute(
            &local_name!("href"),
            DOMString::from_string(value.0),
            can_gc,
        );

        self.set_url();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-origin>
    fn get_origin(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        USVString(match *self.get_url().borrow() {
            // Step 2. If this's url is null, return the empty string.
            None => "".to_owned(),
            // Step 3. Return the serialization of this's url's origin.
            Some(ref url) => url.origin().ascii_serialization(),
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-password>
    fn get_password(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url is null, then return the empty string.
            None => USVString(String::new()),
            // Steps 4. Return url's password.
            Some(ref url) => UrlHelper::Password(url),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-password>
    fn set_password(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null or url cannot have a username/password/port, then return.
            None => return,
            Some(ref url) if url.host().is_none() || url.cannot_be_a_base() => return,
            // Step 4. Set the password, given url and the given value.
            Some(url) => {
                UrlHelper::SetPassword(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 5. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname>
    fn get_pathname(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url is null, then return the empty string.
            None => USVString(String::new()),
            // Steps 4. Return the result of URL path serializing url.
            Some(ref url) => UrlHelper::Pathname(url),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname>
    fn set_pathname(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null or url has an opaque path, then return.
            None => return,
            Some(ref url) if url.cannot_be_a_base() => return,
            // Step 4. Set url's path to the empty list.
            // Step 5. Basic URL parse the given value, with url as url and path start state as state override.
            Some(url) => {
                UrlHelper::SetPathname(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 6. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-port>
    fn get_port(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url or url's port is null, return the empty string.
            None => USVString(String::new()),
            // Step 4. Return url's port, serialized.
            Some(ref url) => UrlHelper::Port(url),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-port>
    fn set_port(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null or url cannot have a username/password/port, then return.
            None => return,
            Some(ref url)
                // https://url.spec.whatwg.org/#cannot-have-a-username-password-port
                if url.host().is_none() || url.cannot_be_a_base() || url.scheme() == "file" =>
            {
                return;
            },
            // Step 4. If the given value is the empty string, then set url's port to null.
            // Step 5. Otherwise, basic URL parse the given value, with url as url and port state as
            // state override.
            Some(url) => {
                UrlHelper::SetPort(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 6. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol>
    fn get_protocol(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        match *self.get_url().borrow() {
            // Step 2. If this's url is null, return ":".
            None => USVString(":".to_owned()),
            // Step 3. Return this's url's scheme, followed by ":".
            Some(ref url) => UrlHelper::Protocol(url),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol>
    fn set_protocol(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 2. If this's url is null, then return.
            None => return,
            // Step 3. Basic URL parse the given value, followed by ":", with this's url as url and
            // scheme start state as state override.
            Some(url) => {
                UrlHelper::SetProtocol(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 4. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-search>
    fn get_search(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        match *self.get_url().borrow() {
            // Step 3. If url is null, or url's query is either null or the empty string, return the
            // empty string.
            // Step 4. Return "?", followed by url's query.
            // Note: This is handled in UrlHelper::Search
            None => USVString(String::new()),
            Some(ref url) => UrlHelper::Search(url),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-search>
    fn set_search(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null, terminate these steps.
            None => return,
            // Step 4. If the given value is the empty string, set url's query to null.
            // Step 5. Otherwise:
            Some(url) => {
                // Note: Inner steps are handled by UrlHelper::SetSearch
                UrlHelper::SetSearch(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 6. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-username>
    fn get_username(&self) -> USVString {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        match *self.get_url().borrow() {
            // Step 2. If this's url is null, return the empty string.
            None => USVString(String::new()),
            // Step 3. Return this's url's username.
            Some(ref url) => UrlHelper::Username(url),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-hyperlink-url-set>
    fn set_url(&self) {
        // Step 1. Set this element's url to null.
        *self.get_url().borrow_mut() = None;

        let attribute = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("href"));

        // Step 2. If this element's href content attribute is absent, then return.
        let Some(attribute) = attribute else {
            return;
        };

        let document = self.owner_document();

        // Step 3. Let url be the result of encoding-parsing a URL given this element's href content
        // attribute's value, relative to this element's node document.
        let url = document.encoding_parse_a_url(&attribute.value());

        // Step 4. If url is not failure, then set this element's url to url.
        if let Ok(url) = url {
            *self.get_url().borrow_mut() = Some(url);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-username>
    fn set_username(&self, value: USVString, can_gc: CanGc) {
        // Step 1. Reinitialize url.
        self.reinitialize_url();

        // Step 2. Let url be this's url.
        let url = match self.get_url().borrow_mut().as_mut() {
            // Step 3. If url is null or url cannot have a username/password/port, then return.
            None => return,
            Some(ref url) if url.host().is_none() || url.cannot_be_a_base() => return,
            // Step 4. Set the username, given url and the given value.
            Some(url) => {
                UrlHelper::SetUsername(url, value);
                DOMString::from(url.as_str())
            },
        };

        // Step 5. Update href.
        self.update_href(url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#update-href>
    fn update_href(&self, url: DOMString, can_gc: CanGc) {
        self.upcast::<Element>()
            .set_string_attribute(&local_name!("href"), url, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#reinitialise-url>
    fn reinitialize_url(&self) {
        // Step 1. If the element's url is non-null, its scheme is "blob", and it has an opaque
        // path, then terminate these steps.
        match *self.get_url().borrow() {
            Some(ref url) if url.scheme() == "blob" && url.cannot_be_a_base() => return,
            _ => (),
        }

        // Step 2. Set the url.
        self.set_url();
    }
}
