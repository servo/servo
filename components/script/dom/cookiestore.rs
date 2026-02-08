/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;

use base::generic_channel::{GenericSend, GenericSender};
use base::id::CookieStoreId;
use cookie::{Cookie, SameSite};
use dom_struct::dom_struct;
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use itertools::Itertools;
use js::jsval::NullValue;
use net_traits::CookieSource::NonHTTP;
use net_traits::{CookieAsyncResponse, CookieData, CoreResourceMsg};
use script_bindings::codegen::GenericBindings::CookieStoreBinding::CookieSameSite;
use script_bindings::script_runtime::CanGc;
use servo_url::ServoUrl;
use time::OffsetDateTime;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CookieStoreBinding::{
    CookieInit, CookieListItem, CookieStoreDeleteOptions, CookieStoreGetOptions, CookieStoreMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::task_source::SendableTaskSource;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableCookieStore {
    // Store an id so that we can send it with requests and the resource thread knows who to respond to
    #[no_trace]
    store_id: CookieStoreId,
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    unregister_channel: GenericSender<CoreResourceMsg>,
}

impl Drop for DroppableCookieStore {
    fn drop(&mut self) {
        let res = self
            .unregister_channel
            .send(CoreResourceMsg::RemoveCookieListener(self.store_id));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        }
    }
}

/// <https://cookiestore.spec.whatwg.org/>
/// CookieStore provides an async API for pages and service workers to access and modify cookies.
/// This requires setting up communication with resource thread's cookie storage that allows for
/// the page to have multiple cookie storage promises in flight at the same time.
#[dom_struct]
pub(crate) struct CookieStore {
    eventtarget: EventTarget,
    #[conditional_malloc_size_of]
    in_flight: DomRefCell<VecDeque<Rc<Promise>>>,
    droppable: DroppableCookieStore,
}

struct CookieListener {
    // TODO:(whatwg/cookiestore#239) The spec is missing details for what task source to use
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
                CookieData::Get(cookie) => {
                    // If list is failure, then reject p with a TypeError and abort these steps.
                    // (There is currently no way for list to result in failure)
                    if let Some(cookie) = cookie {
                        // Otherwise, resolve p with the first item of list.
                        promise.resolve_native(&cookie_to_list_item(cookie.into_inner()), CanGc::note());
                    } else {
                        // If list is empty, then resolve p with null.
                        promise.resolve_native(&NullValue(), CanGc::note());
                    }
                },
                CookieData::GetAll(cookies) => {
                    // If list is failure, then reject p with a TypeError and abort these steps.
                    promise.resolve_native(
                        &cookies
                        .into_iter()
                        .map(|cookie| cookie_to_list_item(cookie.0))
                        .collect_vec(),
                    CanGc::note());
                },
                CookieData::Delete(_) | CookieData::Change(_) | CookieData::Set(_) => {
                    promise.resolve_native(&(), CanGc::note());
                }
            }
        }));
    }
}

impl CookieStore {
    fn new_inherited(unregister_channel: GenericSender<CoreResourceMsg>) -> CookieStore {
        CookieStore {
            eventtarget: EventTarget::new_inherited(),
            in_flight: Default::default(),
            droppable: DroppableCookieStore {
                store_id: CookieStoreId::new(),
                unregister_channel,
            },
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<CookieStore> {
        let store = reflect_dom_object(
            Box::new(CookieStore::new_inherited(
                global.resource_threads().core_thread.clone(),
            )),
            global,
            can_gc,
        );
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

        let res = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::NewCookieListener(
                self.droppable.store_id,
                cookie_sender,
                self.global().creation_url().clone(),
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        }
    }
}

/// <https://cookiestore.spec.whatwg.org/#create-a-cookielistitem>
fn cookie_to_list_item(cookie: Cookie) -> CookieListItem {
    // TODO: Investigate if we need to explicitly UTF-8 decode without BOM here or if thats
    // already being done by cookie-rs or implicitly by using rust strings
    CookieListItem {
        // Let name be the result of running UTF-8 decode without BOM on cookie’s name.
        name: Some(cookie.name().to_string().into()),

        // Let value be the result of running UTF-8 decode without BOM on cookie’s value.
        value: Some(cookie.value().to_string().into()),
    }
}

impl CookieStoreMethods<crate::DomTypeHolder> for CookieStore {
    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-get>
    fn Get(&self, name: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 5. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }

        // 4. Let url be settings’s creation URL.
        let creation_url = global.creation_url();

        let name = CookieStore::normalize(&name);

        // 6. Run the following steps in parallel:
        let res = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::GetCookieDataForUrlAsync(
                self.droppable.store_id,
                creation_url.clone(),
                Some(name),
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 7. Return p.
        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-get-options>
    fn Get_(&self, options: &CookieStoreGetOptions, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 7. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
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

            // 6.2. If this’s relevant global object is a Window object and parsed does not equal url with exclude fragments set to true,
            // then return a promise rejected with a TypeError.
            if let Some(_window) = DomRoot::downcast::<Window>(self.global()) {
                if parsed_url
                    .as_ref()
                    .is_ok_and(|parsed| !parsed.is_equal_excluding_fragments(&creation_url))
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
            if let Ok(url) = parsed_url {
                final_url = url;
            }
        }

        // 6. Run the following steps in parallel:
        let res = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::GetCookieDataForUrlAsync(
                self.droppable.store_id,
                final_url.clone(),
                options.name.clone().map(|val| CookieStore::normalize(&val)),
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-getall>
    fn GetAll(&self, name: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 5. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }
        // 4. Let url be settings’s creation URL.
        let creation_url = global.creation_url();

        // Normalize name here rather than passing the un-nomarlized name around to the resource thread and back
        let name = CookieStore::normalize(&name);

        // 6. Run the following steps in parallel:
        let res =
            self.global()
                .resource_threads()
                .send(CoreResourceMsg::GetAllCookieDataForUrlAsync(
                    self.droppable.store_id,
                    creation_url.clone(),
                    Some(name),
                ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 7. Return p.
        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-getall-options>
    fn GetAll_(&self, options: &CookieStoreGetOptions, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 6. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }

        // 4. Let url be settings’s creation URL.
        let creation_url = global.creation_url();

        let mut final_url = creation_url.clone();

        // 5. If options["url"] is present, then run these steps:
        if let Some(get_url) = &options.url {
            // 5.1. Let parsed be the result of parsing options["url"] with settings’s API base URL.
            let parsed_url = ServoUrl::parse_with_base(Some(&global.api_base_url()), get_url);

            // If this’s relevant global object is a Window object and parsed does not equal url with exclude fragments set to true,
            // then return a promise rejected with a TypeError.
            if let Some(_window) = DomRoot::downcast::<Window>(self.global()) {
                if parsed_url
                    .as_ref()
                    .is_ok_and(|parsed| !parsed.is_equal_excluding_fragments(&creation_url))
                {
                    p.reject_error(
                        Error::Type("URL does not match context".to_string()),
                        can_gc,
                    );
                    return p;
                }
            }

            // 5.3. If parsed’s origin and url’s origin are not the same origin,
            // then return a promise rejected with a TypeError.
            if parsed_url
                .as_ref()
                .is_ok_and(|parsed| creation_url.origin() != parsed.origin())
            {
                p.reject_error(Error::Type("Not same origin".to_string()), can_gc);
                return p;
            }

            // 5.4. Set url to parsed.
            if let Ok(url) = parsed_url {
                final_url = url;
            }
        }

        // 7. Run the following steps in parallel:
        let res =
            self.global()
                .resource_threads()
                .send(CoreResourceMsg::GetAllCookieDataForUrlAsync(
                    self.droppable.store_id,
                    final_url.clone(),
                    options.name.clone().map(|val| CookieStore::normalize(&val)),
                ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 8. Return p
        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-set>
    fn Set(&self, name: USVString, value: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 9. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }

        // From https://cookiestore.spec.whatwg.org/#set-cookie-algorithm
        // Normalize name and value
        // We do this here so we don't have to modify the cookie name/value again
        let name = CookieStore::normalize(&name);
        let value = CookieStore::normalize(&value);

        if !CookieStore::validate_name_and_value(&name, &value) {
            p.reject_error(Error::Type("Invalid cookie".to_string()), can_gc);
            return p;
        }

        // 4. Let url be settings’s creation URL.
        // 5. Let domain be null.
        // 6. Let path be "/".
        // 7. Let sameSite be strict.
        // 8. Let partitioned be false.
        let cookie = Cookie::build((Cow::Owned(name), Cow::Owned(value)))
            .path("/")
            .secure(true)
            .same_site(SameSite::Strict)
            .partitioned(false);
        // TODO: This currently doesn't implement all the "set a cookie" steps which involves
        // additional processing of the name and value

        // 10. Run the following steps in parallel:
        let res = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::SetCookieForUrlAsync(
                self.droppable.store_id,
                self.global().creation_url().clone(),
                Serde(cookie.build()),
                NonHTTP,
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 11. Return p.
        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-set-options>
    fn Set_(&self, options: &CookieInit, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 5. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }

        // 4. Let url be settings’s creation URL.
        let creation_url = global.creation_url();

        // From https://cookiestore.spec.whatwg.org/#set-cookie-algorithm
        // Normalize name and value
        // We do this here so we don't have to modify the cookie name/value again
        let name = CookieStore::normalize(&options.name);
        let value = CookieStore::normalize(&options.value);

        if !CookieStore::validate_name_and_value(&name, &value) {
            p.reject_error(Error::Type("Invalid cookie".to_string()), can_gc);
            return p;
        }

        // 6.1. Let r be the result of running set a cookie with url, options["name"], options["value"],
        // options["expires"], options["domain"], options["path"], options["sameSite"], and options["partitioned"].
        let mut cookie = Cookie::build((Cow::Owned(name), Cow::Owned(value)))
            .path(options.path.0.clone())
            .secure(true)
            .http_only(false)
            .same_site(match options.sameSite {
                CookieSameSite::Lax => SameSite::Lax,
                CookieSameSite::Strict => SameSite::Strict,
                CookieSameSite::None => SameSite::None,
            });
        if let Some(domain) = &options.domain {
            cookie.inner_mut().set_domain(domain.0.clone());
        }
        if let Some(expiry) = options.expires {
            cookie.inner_mut().set_expires(
                OffsetDateTime::from_unix_timestamp((*expiry / 1000.0) as i64).unwrap(),
            );
        }
        // TODO: This currently doesn't implement all the "set a cookie" steps which involves
        // additional processing of the name and value

        // 6. Run the following steps in parallel:
        let res = self
            .global()
            .resource_threads()
            .send(CoreResourceMsg::SetCookieForUrlAsync(
                self.droppable.store_id,
                creation_url.clone(),
                Serde(cookie.build()),
                NonHTTP,
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 7. Return p
        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-delete>
    fn Delete(&self, name: USVString, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 5. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }

        // 6. Run the following steps in parallel:
        // TODO: the spec passes additional parameters to _delete a cookie_ that we don't handle yet
        let res = global
            .resource_threads()
            .send(CoreResourceMsg::DeleteCookieAsync(
                self.droppable.store_id,
                global.creation_url().clone(),
                name.0,
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 7. Return p.
        p
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestore-delete-options>
    fn Delete_(&self, options: &CookieStoreDeleteOptions, can_gc: CanGc) -> Rc<Promise> {
        // 1. Let settings be this’s relevant settings object.
        let global = self.global();

        // 2. Let origin be settings’s origin.
        let origin = global.origin();

        // 5. Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // 3. If origin is an opaque origin, then return a promise rejected with a "SecurityError" DOMException.
        if !origin.is_tuple() {
            p.reject_error(Error::Security(None), can_gc);
            return p;
        }

        // 6. Run the following steps in parallel:
        // TODO: the spec passes additional parameters to _delete a cookie_ that we don't handle yet
        let res = global
            .resource_threads()
            .send(CoreResourceMsg::DeleteCookieAsync(
                self.droppable.store_id,
                global.creation_url().clone(),
                options.name.to_string(),
            ));
        if res.is_err() {
            error!("Failed to send cookiestore message to resource threads");
        } else {
            self.in_flight.borrow_mut().push_back(p.clone());
        }

        // 7. Return p.
        p
    }
}

impl CookieStore {
    /// <https://cookiestore.spec.whatwg.org/#normalize-a-cookie-name-or-value>
    fn normalize(value: &USVString) -> String {
        value.trim_matches([' ', '\t']).into()
    }

    /// <https://cookiestore.spec.whatwg.org/#set-cookie-algorithm>
    fn validate_name_and_value(name: &str, value: &str) -> bool {
        // If name or value contain U+003B (;), any C0 control character except U+0009 TAB, or U+007F DELETE, then return failure.
        if CookieStore::contains_control_characters(name) ||
            CookieStore::contains_control_characters(value)
        {
            return false;
        }

        // If name contains U+003D (=), then return failure.
        if name.contains('=') {
            return false;
        }

        // If name’s length is 0:
        if name.is_empty() {
            // If value contains U+003D (=), then return failure.
            // If value’s length is 0, then return failure.
            if value.contains('=') || value.is_empty() {
                return false;
            }
            // If value, byte-lowercased, starts with `__host-`, `__host-http-`, `__http-`, or `__secure-`, then return failure.
            let lowercased_value = value.to_ascii_lowercase();
            if ["__host-", "__host-http-", "__http-", "__secure-"]
                .iter()
                .any(|prefix| lowercased_value.starts_with(prefix))
            {
                return false;
            }
        }

        // If name, byte-lowercased, starts with `__host-http-` or `__http-`, then return failure.
        let lowercased_name = name.to_ascii_lowercase();
        if lowercased_name.starts_with("__host-http-") || lowercased_name.starts_with("__http-") {
            return false;
        }

        true
    }

    /// If name or value contain U+003B (;), any C0 control character except U+0009 TAB, or U+007F DELETE, then return failure.
    fn contains_control_characters(val: &str) -> bool {
        val.contains(
            |v| matches!(v, '\u{0000}'..='\u{0008}' | '\u{000a}'..='\u{001f}' | '\u{007f}' | ';'),
        )
    }
}
