/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use net_traits::blob_url_store::{get_blob_origin, parse_blob_url};
use net_traits::filemanager_thread::FileManagerThreadMsg;
use net_traits::{CoreResourceMsg, IpcSend};
use profile_traits::ipc;
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::URLBinding::URLMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::globalscope::GlobalScope;
use crate::dom::urlhelper::UrlHelper;
use crate::dom::urlsearchparams::URLSearchParams;

/// <https://url.spec.whatwg.org/#url>
#[dom_struct]
pub struct URL {
    reflector_: Reflector,

    /// <https://url.spec.whatwg.org/#concept-url-url>
    #[no_trace]
    url: DomRefCell<ServoUrl>,

    /// <https://url.spec.whatwg.org/#dom-url-searchparams>
    search_params: MutNullableDom<URLSearchParams>,
}

impl URL {
    fn new_inherited(url: ServoUrl) -> URL {
        URL {
            reflector_: Reflector::new(),
            url: DomRefCell::new(url),
            search_params: Default::default(),
        }
    }

    fn new(global: &GlobalScope, proto: Option<HandleObject>, url: ServoUrl) -> DomRoot<URL> {
        reflect_dom_object_with_proto(Box::new(URL::new_inherited(url)), global, proto)
    }

    pub fn query_pairs(&self) -> Vec<(String, String)> {
        self.url
            .borrow()
            .as_url()
            .query_pairs()
            .into_owned()
            .collect()
    }

    pub fn set_query_pairs(&self, pairs: &[(String, String)]) {
        let mut url = self.url.borrow_mut();

        if pairs.is_empty() {
            url.as_mut_url().set_query(None);
        } else {
            url.as_mut_url()
                .query_pairs_mut()
                .clear()
                .extend_pairs(pairs);
        }
    }
}

#[allow(non_snake_case)]
impl URL {
    /// <https://url.spec.whatwg.org/#constructors>
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        url: USVString,
        base: Option<USVString>,
    ) -> Fallible<DomRoot<URL>> {
        // Step 1. Parse url with base.
        let parsed_base = match base {
            None => None,
            Some(base) => {
                match ServoUrl::parse(&base.0) {
                    Ok(base) => Some(base),
                    Err(error) => {
                        // Step 2. Throw a TypeError if URL parsing fails.
                        return Err(Error::Type(format!("could not parse base: {}", error)));
                    },
                }
            },
        };
        let parsed_url = match ServoUrl::parse_with_base(parsed_base.as_ref(), &url.0) {
            Ok(url) => url,
            Err(error) => {
                // Step 2. Throw a TypeError if URL parsing fails.
                return Err(Error::Type(format!("could not parse URL: {}", error)));
            },
        };

        // Skip the steps below.
        // Instead of constructing a new `URLSearchParams` object here, construct it
        // on-demand inside `URL::SearchParams`.
        //
        // Step 3. Let query be parsedURL’s query.
        // Step 5. Set this’s query object to a new URLSearchParams object.
        // Step 6. Initialize this’s query object with query.
        // Step 7. Set this’s query object’s URL object to this.

        // Step 4. Set this’s URL to parsedURL.
        Ok(URL::new(global, proto, parsed_url))
    }

    /// <https://url.spec.whatwg.org/#dom-url-canparse>
    pub fn CanParse(_global: &GlobalScope, url: USVString, base: Option<USVString>) -> bool {
        // Step 1.
        let parsed_base = match base {
            None => None,
            Some(base) => match ServoUrl::parse(&base.0) {
                Ok(base) => Some(base),
                Err(_) => {
                    // Step 2.1
                    return false;
                },
            },
        };
        // Step 2.2, 3
        ServoUrl::parse_with_base(parsed_base.as_ref(), &url.0).is_ok()
    }

    /// <https://url.spec.whatwg.org/#dom-url-parse>
    pub fn Parse(
        global: &GlobalScope,
        url: USVString,
        base: Option<USVString>,
    ) -> Option<DomRoot<URL>> {
        // Step 1: Let parsedURL be the result of running the API URL parser on url with base,
        // if given.
        let parsed_base = base.and_then(|base| ServoUrl::parse(base.0.as_str()).ok());
        let parsed_url = ServoUrl::parse_with_base(parsed_base.as_ref(), &url.0);

        // Step 2: If parsedURL is failure, then return null.
        // Step 3: Let url be a new URL object.
        // Step 4: Initialize url with parsedURL.
        // Step 5: Return url.

        // These steps are all handled while mapping the Result to an Option<ServoUrl>.
        // Regarding initialization, the same condition should apply here as stated in the comments
        // in Self::Constructor above - construct it on-demand inside `URL::SearchParams`.
        Some(URL::new(global, None, parsed_url.ok()?))
    }

    /// <https://w3c.github.io/FileAPI/#dfn-createObjectURL>
    pub fn CreateObjectURL(global: &GlobalScope, blob: &Blob) -> DOMString {
        // XXX: Second field is an unicode-serialized Origin, it is a temporary workaround
        //      and should not be trusted. See issue https://github.com/servo/servo/issues/11722
        let origin = get_blob_origin(&global.get_url());

        let id = blob.get_blob_url_id();

        DOMString::from(URL::unicode_serialization_blob_url(&origin, &id))
    }

    /// <https://w3c.github.io/FileAPI/#dfn-revokeObjectURL>
    pub fn RevokeObjectURL(global: &GlobalScope, url: DOMString) {
        // If the value provided for the url argument is not a Blob URL OR
        // if the value provided for the url argument does not have an entry in the Blob URL Store,
        // this method call does nothing. User agents may display a message on the error console.
        let origin = get_blob_origin(&global.get_url());

        if let Ok(url) = ServoUrl::parse(&url) {
            if url.fragment().is_none() && origin == get_blob_origin(&url) {
                if let Ok((id, _)) = parse_blob_url(&url) {
                    let resource_threads = global.resource_threads();
                    let (tx, rx) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
                    let msg = FileManagerThreadMsg::RevokeBlobURL(id, origin, tx);
                    let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));

                    let _ = rx.recv().unwrap();
                }
            }
        }
    }

    /// <https://w3c.github.io/FileAPI/#unicodeSerializationOfBlobURL>
    fn unicode_serialization_blob_url(origin: &str, id: &Uuid) -> String {
        // Step 1, 2
        let mut result = "blob:".to_string();

        // Step 3
        result.push_str(origin);

        // Step 4
        result.push('/');

        // Step 5
        result.push_str(&id.to_string());

        result
    }
}

#[allow(non_snake_case)]
impl URLMethods for URL {
    /// <https://url.spec.whatwg.org/#dom-url-hash>
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-hash>
    fn SetHash(&self, value: USVString) {
        UrlHelper::SetHash(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-host>
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-host>
    fn SetHost(&self, value: USVString) {
        UrlHelper::SetHost(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-hostname>
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-hostname>
    fn SetHostname(&self, value: USVString) {
        UrlHelper::SetHostname(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-href>
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-href>
    fn SetHref(&self, value: USVString) -> ErrorResult {
        match ServoUrl::parse(&value.0) {
            Ok(url) => {
                *self.url.borrow_mut() = url;
                self.search_params.set(None); // To be re-initialized in the SearchParams getter.
                Ok(())
            },
            Err(error) => Err(Error::Type(format!("could not parse URL: {}", error))),
        }
    }

    /// <https://url.spec.whatwg.org/#dom-url-password>
    fn Password(&self) -> USVString {
        UrlHelper::Password(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-password>
    fn SetPassword(&self, value: USVString) {
        UrlHelper::SetPassword(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-pathname>
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-pathname>
    fn SetPathname(&self, value: USVString) {
        UrlHelper::SetPathname(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-port>
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-port>
    fn SetPort(&self, value: USVString) {
        UrlHelper::SetPort(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-protocol>
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-protocol>
    fn SetProtocol(&self, value: USVString) {
        UrlHelper::SetProtocol(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-origin>
    fn Origin(&self) -> USVString {
        UrlHelper::Origin(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-search>
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-search>
    fn SetSearch(&self, value: USVString) {
        UrlHelper::SetSearch(&mut self.url.borrow_mut(), value);
        if let Some(search_params) = self.search_params.get() {
            search_params.set_list(self.query_pairs());
        }
    }

    /// <https://url.spec.whatwg.org/#dom-url-searchparams>
    fn SearchParams(&self) -> DomRoot<URLSearchParams> {
        self.search_params
            .or_init(|| URLSearchParams::new(&self.global(), Some(self)))
    }

    /// <https://url.spec.whatwg.org/#dom-url-username>
    fn Username(&self) -> USVString {
        UrlHelper::Username(&self.url.borrow())
    }

    /// <https://url.spec.whatwg.org/#dom-url-username>
    fn SetUsername(&self, value: USVString) {
        UrlHelper::SetUsername(&mut self.url.borrow_mut(), value);
    }

    /// <https://url.spec.whatwg.org/#dom-url-tojson>
    fn ToJSON(&self) -> USVString {
        self.Href()
    }
}
