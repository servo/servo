/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::URLBinding::{self, URLMethods};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::blob::Blob;
use dom::urlhelper::UrlHelper;
use dom::urlsearchparams::URLSearchParams;
use ipc_channel::ipc;
use net_traits::IpcSend;
use net_traits::blob_url_store::{get_blob_origin, parse_blob_url};
use net_traits::filemanager_thread::{SelectedFileId, FileManagerThreadMsg};
use std::borrow::ToOwned;
use std::default::Default;
use url::quirks::domain_to_unicode;
use url::{Host, Url};
use uuid::Uuid;

// https://url.spec.whatwg.org/#url
#[dom_struct]
pub struct URL {
    reflector_: Reflector,

    // https://url.spec.whatwg.org/#concept-url-url
    url: DOMRefCell<Url>,

    // https://url.spec.whatwg.org/#dom-url-searchparams
    search_params: MutNullableHeap<JS<URLSearchParams>>,
}

impl URL {
    fn new_inherited(url: Url) -> URL {
        URL {
            reflector_: Reflector::new(),
            url: DOMRefCell::new(url),
            search_params: Default::default(),
        }
    }

    pub fn new(global: GlobalRef, url: Url) -> Root<URL> {
        reflect_dom_object(box URL::new_inherited(url),
                           global, URLBinding::Wrap)
    }

    pub fn query_pairs(&self) -> Vec<(String, String)> {
        self.url.borrow().query_pairs().into_owned().collect()
    }

    pub fn set_query_pairs(&self, pairs: &[(String, String)]) {
        let mut url = self.url.borrow_mut();
        url.query_pairs_mut().clear().extend_pairs(pairs);
    }
}

impl URL {
    // https://url.spec.whatwg.org/#constructors
    pub fn Constructor(global: GlobalRef, url: USVString,
                       base: Option<USVString>)
                       -> Fallible<Root<URL>> {
        let parsed_base = match base {
            None => {
                // Step 1.
                None
            },
            Some(base) =>
                // Step 2.1.
                match Url::parse(&base.0) {
                    Ok(base) => Some(base),
                    Err(error) => {
                        // Step 2.2.
                        return Err(Error::Type(format!("could not parse base: {}", error)));
                    }
                }
        };
        // Step 3.
        let parsed_url = match Url::options().base_url(parsed_base.as_ref()).parse(&url.0) {
            Ok(url) => url,
            Err(error) => {
                // Step 4.
                return Err(Error::Type(format!("could not parse URL: {}", error)));
            }
        };
        // Step 5: Skip (see step 8 below).
        // Steps 6-7.
        let result = URL::new(global, parsed_url);
        // Step 8: Instead of construcing a new `URLSearchParams` object here, construct it
        //         on-demand inside `URL::SearchParams`.
        // Step 9.
        Ok(result)
    }

    // https://url.spec.whatwg.org/#dom-url-domaintoasciidomain
    pub fn DomainToASCII(_: GlobalRef, origin: USVString) -> USVString {
        // Step 1.
        let ascii_domain = Host::parse(&origin.0);
        if let Ok(Host::Domain(string)) = ascii_domain {
            // Step 3.
            USVString(string.to_owned())
        } else {
            // Step 2.
            USVString("".to_owned())
        }
    }

    pub fn DomainToUnicode(_: GlobalRef, origin: USVString) -> USVString {
        USVString(domain_to_unicode(&origin.0))
    }

    // https://w3c.github.io/FileAPI/#dfn-createObjectURL
    pub fn CreateObjectURL(global: GlobalRef, blob: &Blob) -> DOMString {
        /// XXX: Second field is an unicode-serialized Origin, it is a temporary workaround
        ///      and should not be trusted. See issue https://github.com/servo/servo/issues/11722
        let origin = get_blob_origin(&global.get_url());

        if blob.IsClosed() {
            // Generate a dummy id
            let id = Uuid::new_v4().simple().to_string();
            return DOMString::from(URL::unicode_serialization_blob_url(&origin, &id));
        }

        let id = blob.get_blob_url_id();

        DOMString::from(URL::unicode_serialization_blob_url(&origin, &id.0))
    }

    // https://w3c.github.io/FileAPI/#dfn-revokeObjectURL
    pub fn RevokeObjectURL(global: GlobalRef, url: DOMString) {
        /*
            If the url refers to a Blob that has a readability state of CLOSED OR
            if the value provided for the url argument is not a Blob URL, OR
            if the value provided for the url argument does not have an entry in the Blob URL Store,

            this method call does nothing. User agents may display a message on the error console.

            NOTE: The first step is unnecessary, since closed blobs do not exist in the store
        */
        let origin = get_blob_origin(&global.get_url());

        if let Ok(url) = Url::parse(&url) {
             if let Ok((id, _, _)) = parse_blob_url(&url) {
                let filemanager = global.resource_threads().sender();
                let id = SelectedFileId(id.simple().to_string());
                let (tx, rx) = ipc::channel().unwrap();
                let msg = FileManagerThreadMsg::RevokeBlobURL(id, origin, tx);
                let _ = filemanager.send(msg);

                let _ = rx.recv().unwrap();
            }
        }
    }

    // https://w3c.github.io/FileAPI/#unicodeSerializationOfBlobURL
    fn unicode_serialization_blob_url(origin: &str, id: &str) -> String {
        // Step 1, 2
        let mut result = "blob:".to_string();

        // Step 3
        result.push_str(origin);

        // Step 4
        result.push('/');

        // Step 5
        result.push_str(id);

        result
    }
}

impl URLMethods for URL {
    // https://url.spec.whatwg.org/#dom-url-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-hash
    fn SetHash(&self, value: USVString) {
        UrlHelper::SetHash(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-host
    fn SetHost(&self, value: USVString) {
        UrlHelper::SetHost(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-hostname
    fn SetHostname(&self, value: USVString) {
        UrlHelper::SetHostname(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-href
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-href
    fn SetHref(&self, value: USVString) -> ErrorResult {
        match Url::parse(&value.0) {
            Ok(url) => {
                *self.url.borrow_mut() = url;
                self.search_params.set(None);  // To be re-initialized in the SearchParams getter.
                Ok(())
            },
            Err(error) => {
                Err(Error::Type(format!("could not parse URL: {}", error)))
            },
        }
    }

    // https://url.spec.whatwg.org/#dom-url-password
    fn Password(&self) -> USVString {
        UrlHelper::Password(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-password
    fn SetPassword(&self, value: USVString) {
        UrlHelper::SetPassword(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-pathname
    fn SetPathname(&self, value: USVString) {
        UrlHelper::SetPathname(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-port
    fn SetPort(&self, value: USVString) {
        UrlHelper::SetPort(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-protocol
    fn SetProtocol(&self, value: USVString) {
        UrlHelper::SetProtocol(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-url-origin
    fn Origin(&self) -> USVString {
        UrlHelper::Origin(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-search
    fn SetSearch(&self, value: USVString) {
        UrlHelper::SetSearch(&mut self.url.borrow_mut(), value);
        if let Some(search_params) = self.search_params.get() {
            search_params.set_list(self.url.borrow().query_pairs().into_owned().collect());
        }
    }

    // https://url.spec.whatwg.org/#dom-url-searchparams
    fn SearchParams(&self) -> Root<URLSearchParams> {
        self.search_params.or_init(|| URLSearchParams::new(self.global().r(), Some(self)))
    }

    // https://url.spec.whatwg.org/#dom-url-href
    fn Stringifier(&self) -> DOMString {
        DOMString::from(self.Href().0)
    }

    // https://url.spec.whatwg.org/#dom-url-username
    fn Username(&self) -> USVString {
        UrlHelper::Username(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-url-username
    fn SetUsername(&self, value: USVString) {
        UrlHelper::SetUsername(&mut self.url.borrow_mut(), value);
    }
}
