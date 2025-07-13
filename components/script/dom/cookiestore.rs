/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;

use base::id::CookieStoreId;
use cookie::Expiration::DateTime;
use cookie::{Cookie, SameSite};
use dom_struct::dom_struct;
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::CookieSource::NonHTTP;
use net_traits::{CookieAsyncResponse, CookieData, CoreResourceMsg, IpcSend};
use script_bindings::script_runtime::CanGc;
use servo_url::ServoUrl;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CookieStoreBinding::{
    CookieInit, CookieListItem, CookieSameSite, CookieStoreDeleteOptions, CookieStoreGetOptions,
    CookieStoreMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::task_source::SendableTaskSource;

/// <https://wicg.github.io/cookie-store/>
/// CookieStore provides an async API for pages and service workers to access and modify cookies.
/// This requires setting up communication with resource thread's cookie storage that allows for
/// the page to have multiple cookie storage promises in flight at the same time.
#[dom_struct]
pub(crate) struct CookieStore {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "Rc"]
    in_flight: DomRefCell<VecDeque<Rc<Promise>>>,
    // Store an id so that we can send it with requests and the resource thread knows who to respond to
    #[no_trace]
    store_id: CookieStoreId,
}

struct CookieListener {
    task_source: SendableTaskSource,
    context: Trusted<CookieStore>,
}

impl CookieListener {
    pub(crate) fn handle(&self, message: CookieAsyncResponse) {
        let context = self.context.clone();
        self.task_source.queue(task!(cookie_message: move || {
            let Some(promise) = context.root().in_flight.borrow_mut().pop_front() else {
                warn!("No promise exists for cookie store response");
                return;
            };
            match message.data {
                CookieData::Get(cookies, name) => {
                    // 3. For each cookie in cookie-list, run these steps:
                    // 3.1. Assert: cookie’s http-only-flag is false.
                    // 3.2. If name is given, then run these steps:
                    // 3.2.1. Let cookieName be the result of running UTF-8 decode without BOM on cookie’s name.
                    // 3.2.2. If cookieName does not equal name, then continue.
                    let cookie = cookies.into_iter().find(|c| c.name() == name);
                    if let Some(cookie) = cookie {
                        promise.resolve_native(&cookie_to_list_item(cookie.into_inner()), CanGc::note());
                    } else {
                        promise.resolve_native(&(), CanGc::note());
                    }
                },
                _ => {promise.resolve_native(&(), CanGc::note());}
            }
        }));
    }
}

impl CookieStore {
    fn new_inherited() -> CookieStore {
        CookieStore {
            eventtarget: EventTarget::new_inherited(),
            in_flight: Default::default(),
            store_id: CookieStoreId::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<CookieStore> {
        let store = reflect_dom_object(Box::new(CookieStore::new_inherited()), global, can_gc);
        store.setup_route();
        store
    }

    fn setup_route(&self) {
        let (cookie_sender, cookie_receiver) = ipc::channel().expect("ipc channel failure");

        let context = Trusted::new(self);
        let cs_listener = CookieListener {
            task_source: self
                .global()
                .task_manager()
                .dom_manipulation_task_source()
                .to_sendable(),
            context,
        };

        ROUTER.add_typed_route(
            cookie_receiver,
            Box::new(move |message| match message {
                Ok(msg) => cs_listener.handle(msg),
                Err(err) => warn!("Error receiving a CookieStore message: {:?}", err),
            }),
        );

        let _ = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::NewCookieListener(
                self.store_id,
                cookie_sender,
                self.global().creation_url().clone(),
            ));
    }

    fn query_cookies(&self, url: &ServoUrl, name: Option<USVString>) {
        // 1. Perform the steps defined in Cookies § Retrieval Model to compute the "cookie-string from a given cookie
        // store" with url as request-uri. The cookie-string itself is ignored, but the intermediate cookie-list is
        // used in subsequent steps.
        // For the purposes of the steps, the cookie-string is being generated for a "non-HTTP" API.
        let _ = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::GetCookiesDataForUrlAsync(
                self.store_id,
                url.clone(),
                name.map_or("".to_owned(), |val| val.0),
                NonHTTP,
            ));
    }
}

/// <https://wicg.github.io/cookie-store/#create-a-cookielistitem>
fn cookie_to_list_item(cookie: Cookie) -> CookieListItem {
    CookieListItem {
        // Let domain be the result of running UTF-8 decode without BOM on cookie’s domain.
        domain: cookie
            .domain()
            .map(|domain| Some(domain.to_string().into())),

        // Let expires be cookie’s expiry-time (as a timestamp).
        expires: match cookie.expires() {
            None | Some(cookie::Expiration::Session) => None,
            Some(DateTime(time)) => Some(Some(Finite::wrap((time.unix_timestamp() * 1000) as f64))),
        },

        // Let name be the result of running UTF-8 decode without BOM on cookie’s name.
        name: Some(cookie.name().to_string().into()),

        // Let partitioned be a boolean indicating that the user agent supports cookie partitioning and that i
        // that cookie has a partition key.
        partitioned: Some(false), // Do we support partitioning? Spec says true only if UA supports it

        // Let path be the result of running UTF-8 decode without BOM on cookie’s path.
        path: cookie.path().map(|path| path.to_string().into()),

        sameSite: match cookie.same_site() {
            Some(SameSite::None) => Some(CookieSameSite::None),
            Some(SameSite::Lax) => Some(CookieSameSite::Lax),
            Some(SameSite::Strict) => Some(CookieSameSite::Strict),
            None => None, // The spec doesnt handle this case, which implies the default of Lax?
        },

        // Let secure be cookie’s secure-only-flag.
        secure: cookie.secure(),

        // Let value be the result of running UTF-8 decode without BOM on cookie’s value.
        value: Some(cookie.value().to_string().into()),
    }
}

impl CookieStoreMethods<crate::DomTypeHolder> for CookieStore {
    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-get>
    fn Get(&self, name: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 5. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security, can_gc);
            return p;
        }
        // 4. Let url be settings’s creation URL.
        let creation_url = global.creation_url();

        self.in_flight.borrow_mut().push_back(p.clone());

        // 6. Run the following steps in parallel:
        // 6.1. Let list be the results of running query cookies with url and name.
        // 6.2. If list is failure, then reject p with a TypeError and abort these steps.
        // 6.3. If list is empty, then resolve p with null.
        // 6.4. Otherwise, resolve p with the first item of list.
        self.query_cookies(creation_url, Some(name));

        // 7. Return p.
        p
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-get-options>
    fn Get_(&self, options: &CookieStoreGetOptions, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 7. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security, can_gc);
            return p;
        }

        // 4. Let url be settings’s creation URL.
        let creation_url = global.creation_url();

        // 5. If options is empty, then return a promise rejected with a TypeError.
        // "is empty" is not strictly defined anywhere in the spec but the only value we require here is "url"
        if options.url.is_none() && options.name.is_none() {
            p.reject_error(Error::Type("Options cannot be empty".to_string()), can_gc);
            return p;
        }

        let mut final_url = creation_url.clone();
        // 6. If options["url"] is present, then run these steps:
        if let Some(get_url) = &options.url {
            // 6.1. Let parsed be the result of parsing options["url"] with settings’s API base URL.
            let parsed_url = ServoUrl::parse_with_base(Some(&global.api_base_url()), get_url);

            // 6.2. If this’s relevant global object is a Window object and parsed does not equal url,
            // then return a promise rejected with a TypeError.
            if let Some(_window) = DomRoot::downcast::<Window>(self.global()) {
                if parsed_url
                    .as_ref()
                    .is_ok_and(|parsed| parsed.as_url() != creation_url.as_url())
                {
                    p.reject_error(
                        Error::Type("URL does not match context".to_string()),
                        can_gc,
                    );
                    return p;
                }
            }

            // 6.3. If parsed’s origin and url’s origin are not the same origin,
            // then return a promise rejected with a TypeError.
            if parsed_url
                .as_ref()
                .is_ok_and(|parsed| creation_url.origin() != parsed.origin())
            {
                p.reject_error(Error::Type("Not same origin".to_string()), can_gc);
                return p;
            }

            // 6.4. Set url to parsed.
            if parsed_url.is_ok() {
                final_url = parsed_url.unwrap();
            }
        }

        self.in_flight.borrow_mut().push_back(p.clone());

        self.query_cookies(&final_url, options.name.clone());

        p
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-getall>
    fn GetAll(&self, _name: USVString, _can_gc: CanGc) -> Rc<Promise> {
        todo!();
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-getall-options>
    fn GetAll_(&self, _options: &CookieStoreGetOptions, _can_gc: CanGc) -> Rc<Promise> {
        todo!();
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-set>
    fn Set(&self, name: USVString, value: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 7. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security, can_gc);
            return p;
        }

        let cookie = Cookie::build((Cow::Owned(name.to_string()), Cow::Owned(value.to_string())))
            .path("/")
            .secure(true)
            .same_site(SameSite::Strict)
            .partitioned(false);

        self.in_flight.borrow_mut().push_back(p.clone());

        let _ = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::SetCookieForUrlAsync(
                self.store_id,
                self.global().creation_url().clone(),
                Serde(cookie.build()),
                NonHTTP,
            ));
        p
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-set-options>
    fn Set_(&self, _options: &CookieInit, _can_gc: CanGc) -> Rc<Promise> {
        todo!();
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-delete>
    fn Delete(&self, name: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 7. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security, can_gc);
            return p;
        }

        self.in_flight.borrow_mut().push_back(p.clone());
        let _ = global
            .resource_threads()
            .send(CoreResourceMsg::DeleteCookieAsync(
                self.store_id,
                global.creation_url().clone(),
                name.0,
            ));

        p
    }

    /// <https://wicg.github.io/cookie-store/#dom-cookiestore-delete-options>
    fn Delete_(&self, _options: &CookieStoreDeleteOptions, _can_gc: CanGc) -> Rc<Promise> {
        todo!();
    }

    // https://wicg.github.io/cookie-store/#dom-cookiestore-onchange
    event_handler!(change, GetOnchange, SetOnchange);
}
