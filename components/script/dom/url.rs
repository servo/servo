/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::URLBinding::{self, URLMethods};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::blob::Blob;
use dom::globalscope::GlobalScope;
use dom::urlhelper::UrlHelper;
use dom::urlsearchparams::URLSearchParams;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use net_traits::{CoreResourceMsg, IpcSend};
use net_traits::blob_url_store::{get_blob_origin, parse_blob_url};
use net_traits::filemanager_thread::FileManagerThreadMsg;
use servo_url::ServoUrl;
use std::default::Default;
use uuid::Uuid;

// https://url.spec.whatwg.org/#url
#[dom_struct]
pub struct URL {
    reflector_: Reflector,

    // https://url.spec.whatwg.org/#concept-url-url
    url: DOMRefCell<ServoUrl>,

    // https://url.spec.whatwg.org/#dom-url-searchparams
    search_params: MutNullableJS<URLSearchParams>,
}

impl URL {
    fn new_inherited(url: ServoUrl) -> URL {
        URL {
            reflector_: Reflector::new(),
            url: DOMRefCell::new(url),
            search_params: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope, url: ServoUrl) -> Root<URL> {
        reflect_dom_object(box URL::new_inherited(url),
                           global, URLBinding::Wrap)
    }

    pub fn query_pairs(&self) -> Vec<(String, String)> {
        self.url.borrow().as_url().unwrap().query_pairs().into_owned().collect()
    }

    pub fn set_query_pairs(&self, pairs: &[(String, String)]) {
        if let Some(ref mut url) = self.url.borrow_mut().as_mut_url() {
            url.query_pairs_mut().clear().extend_pairs(pairs);
        }
    }
}

impl URL {
    // https://url.spec.whatwg.org/#constructors
    pub fn Constructor(global: &GlobalScope, url: USVString,
                       base: Option<USVString>)
                       -> Fallible<Root<URL>> {
        let parsed_base = match base {
            None => {
                // Step 1.
                None
            },
            Some(base) =>
                // Step 2.1.
                match ServoUrl::parse(&base.0) {
                    Ok(base) => Some(base),
                    Err(error) => {
                        // Step 2.2.
                        return Err(Error::Type(format!("could not parse base: {}", error)));
                    }
                }
        };
        // Step 3.
        let parsed_url = match ServoUrl::parse_with_base(parsed_base.as_ref(), &url.0) {
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

    // https://w3c.github.io/FileAPI/#dfn-createObjectURL
    pub fn CreateObjectURL(global: &GlobalScope, blob: &Blob) -> DOMString {
        /// XXX: Second field is an unicode-serialized Origin, it is a temporary workaround
        ///      and should not be trusted. See issue https://github.com/servo/servo/issues/11722
        let origin = get_blob_origin(&global.get_url());

        let id = blob.get_blob_url_id();

        DOMString::from(URL::unicode_serialization_blob_url(&origin, &id))
    }

    // https://w3c.github.io/FileAPI/#dfn-revokeObjectURL
    pub fn RevokeObjectURL(global: &GlobalScope, url: DOMString) {
        /*
            If the value provided for the url argument is not a Blob URL OR
            if the value provided for the url argument does not have an entry in the Blob URL Store,

            this method call does nothing. User agents may display a message on the error console.
        */
        let origin = get_blob_origin(&global.get_url());

        if let Ok(url) = ServoUrl::parse(&url) {
             if let Ok((id, _)) = parse_blob_url(&url) {
                let resource_threads = global.resource_threads();
                let (tx, rx) = ipc::channel().unwrap();
                let msg = FileManagerThreadMsg::RevokeBlobURL(id, origin, tx);
                let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));

                let _ = rx.recv().unwrap();
            }
        }
    }

    // https://w3c.github.io/FileAPI/#unicodeSerializationOfBlobURL
    fn unicode_serialization_blob_url(origin: &str, id: &Uuid) -> String {
        // Step 1, 2
        let mut result = "blob:".to_string();

        // Step 3
        result.push_str(origin);

        // Step 4
        result.push('/');

        // Step 5
        result.push_str(&id.simple().to_string());

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
        match ServoUrl::parse(&value.0) {
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
            search_params.set_list(self.query_pairs());
        }
    }

    // https://url.spec.whatwg.org/#dom-url-searchparams
    fn SearchParams(&self) -> Root<URLSearchParams> {
        self.search_params.or_init(|| {
            URLSearchParams::new(&self.global(), Some(self))
        })
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
