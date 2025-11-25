/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::rc::Rc;

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::CredentialsContainerBinding::{
    CredentialCreationOptions, CredentialRequestOptions,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};

use crate::dom::bindings::codegen::Bindings::CredentialsContainerBinding::CredentialsContainerMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::credentialmanagement::credential::Credential;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

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

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<CredentialsContainer> {
        reflect_dom_object(
            Box::new(CredentialsContainer::new_inherited()),
            global,
            can_gc,
        )
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-request-a-credential>
    fn request_credential(
        &self,
        options: &CredentialRequestOptions<DomTypeHolder>,
    ) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        let global = self.global();
        // Step 2. Assert: settings is a secure context.
        assert!(global.is_secure_context());
        // Step 3. Let document be settings’s relevant global object's associated Document.
        let document = global.as_window().Document();
        // Step 4. If document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !document.is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        // Step 5. If options.signal is aborted, then return a promise rejected with options.signal’s abort reason.
        if options.signal.as_ref().is_some_and(|s| s.aborted()) {
            return Err(Error::Abort(None));
        }
        Err(Error::NotSupported(None))
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-store-a-credential>
    fn store_credential(&self, _credential: &Credential) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        let global = self.global();
        // Step 2. Assert: settings is a secure context.
        assert!(global.is_secure_context());
        // Step 3. If settings’s relevant global object's associated Document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !global.as_window().Document().is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        Err(Error::NotSupported(None))
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-create-a-credential>
    fn create_credential(
        &self,
        _options: &CredentialCreationOptions<DomTypeHolder>,
    ) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        let global = self.global();
        // Step 2. Assert: settings is a secure context.
        assert!(global.is_secure_context());
        // Step 3. Let global be settings’ global object.
        // Step 4. Let document be the relevant global object’s associated Document.
        let document = global.as_window().Document();
        // Step 5. If document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !document.is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        Err(Error::NotSupported(None))
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-prevent-silent-access>
    fn prevent_silent_access(&self) -> Fallible<Rc<Promise>> {
        let qlobal = self.global();
        // Step 1. Let origin be settings’ origin.
        let _origin = qlobal.origin();
        // Step 2. If settings’s relevant global object’s associated Document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !qlobal.as_window().Document().is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        // TODO: Step 3. Let p be a new promise.
        // TODO: Step 4. Run the following seps in parallel:
        // TODO: Step 4.1. Set origin’s prevent silent access flag in the credential store.
        // TODO: Step 4.2. Resolve p with undefined.
        // TODO: Step 5. Return p.
        Err(Error::NotSupported(None))
    }
}

impl CredentialsContainerMethods<DomTypeHolder> for CredentialsContainer {
    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-get>
    fn Get(&self, options: &CredentialRequestOptions<DomTypeHolder>) -> Fallible<Rc<Promise>> {
        self.request_credential(options)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-store>
    fn Store(&self, credential: &Credential) -> Fallible<Rc<Promise>> {
        self.store_credential(credential)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-create>
    fn Create(&self, options: &CredentialCreationOptions<DomTypeHolder>) -> Fallible<Rc<Promise>> {
        self.create_credential(options)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-preventsilentaccess>
    fn PreventSilentAccess(&self) -> Fallible<Rc<Promise>> {
        self.prevent_silent_access()
    }
}
