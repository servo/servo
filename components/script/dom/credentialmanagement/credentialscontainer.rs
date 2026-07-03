/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use script_bindings::codegen::GenericBindings::CredentialsContainerBinding::{
    CredentialCreationOptions, CredentialRequestOptions,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::CredentialsContainerBinding::CredentialsContainerMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::root::DomRoot;
use crate::dom::credentialmanagement::credential::Credential;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;

#[dom_struct]
pub(crate) struct CredentialsContainer {
    reflector_: Reflector,
}

impl CredentialsContainer {
    pub(crate) fn new_inherited() -> CredentialsContainer {
        CredentialsContainer {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<CredentialsContainer> {
        reflect_dom_object_with_cx(Box::new(CredentialsContainer::new_inherited()), global, cx)
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-request-a-credential>
    fn request_credential(
        &self,
        cx: &mut CurrentRealm,
        options: &CredentialRequestOptions<DomTypeHolder>,
    ) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        let global = GlobalScope::from_current_realm(cx);
        // Step 2. Assert: settings is a secure context.
        assert!(global.is_secure_context());
        // Step 3. Let document be settings’s relevant global object's associated Document.
        let document = global.as_window().Document();

        let promise = Promise::new_in_realm(cx);
        // Step 4. If document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !document.is_fully_active() {
            promise.reject_error(cx, Error::InvalidState(None));
            return Ok(promise);
        }
        // Step 5. If options.signal is aborted, then return a promise rejected with options.signal’s abort reason.
        if options.signal.as_ref().is_some_and(|s| s.aborted()) {
            promise.reject_error(cx, Error::Abort(None));
            return Ok(promise);
        }
        promise.reject_error(cx, Error::NotSupported(None));
        Ok(promise)
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-store-a-credential>
    fn store_credential(
        &self,
        cx: &mut CurrentRealm,
        _credential: &Credential,
    ) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        let global = GlobalScope::from_current_realm(cx);
        // Step 2. Assert: settings is a secure context.
        assert!(global.is_secure_context());

        let promise = Promise::new_in_realm(cx);
        // Step 3. If settings’s relevant global object's associated Document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !global.as_window().Document().is_fully_active() {
            promise.reject_error(cx, Error::InvalidState(None));
            return Ok(promise);
        }
        promise.reject_error(cx, Error::NotSupported(None));
        Ok(promise)
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-create-a-credential>
    fn create_credential(
        &self,
        cx: &mut CurrentRealm,
        _options: &CredentialCreationOptions<DomTypeHolder>,
    ) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        let global = GlobalScope::from_current_realm(cx);
        // Step 2. Assert: settings is a secure context.
        assert!(global.is_secure_context());
        // Step 3. Let global be settings’ global object.
        // Step 4. Let document be the relevant global object’s associated Document.
        let document = global.as_window().Document();

        let promise = Promise::new_in_realm(cx);
        // Step 5. If document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !document.is_fully_active() {
            promise.reject_error(cx, Error::InvalidState(None));
            return Ok(promise);
        }
        promise.reject_error(cx, Error::NotSupported(None));
        Ok(promise)
    }
}

impl CredentialsContainerMethods<DomTypeHolder> for CredentialsContainer {
    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-get>
    fn Get(
        &self,
        cx: &mut CurrentRealm,
        options: &CredentialRequestOptions<DomTypeHolder>,
    ) -> Fallible<Rc<Promise>> {
        self.request_credential(cx, options)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-store>
    fn Store(&self, cx: &mut CurrentRealm, credential: &Credential) -> Fallible<Rc<Promise>> {
        self.store_credential(cx, credential)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-create>
    fn Create(
        &self,
        cx: &mut CurrentRealm,
        options: &CredentialCreationOptions<DomTypeHolder>,
    ) -> Fallible<Rc<Promise>> {
        self.create_credential(cx, options)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-preventsilentaccess>
    fn PreventSilentAccess(&self, cx: &mut CurrentRealm) -> Fallible<Rc<Promise>> {
        let promise = Promise::new_in_realm(cx);
        promise.reject_error(cx, Error::NotSupported(None));
        Ok(promise)
    }
}
