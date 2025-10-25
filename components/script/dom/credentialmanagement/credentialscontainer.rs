/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::CredentialsContainerBinding::{
    CredentialCreationOptions, CredentialRequestOptions,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;

use crate::dom::bindings::codegen::Bindings::CredentialsContainerBinding::CredentialsContainerMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::credentialmanagement::credential::Credential;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;
use crate::test::TrustedPromise;

#[derive(Serialize, Deserialize)]
struct OriginData {
    passwords: HashMap<String, String>,
}

enum CredentialInterfaceObject {
    Password,
}

/// <https://www.w3.org/TR/credential-management-1/#credentialrequestoptions-relevant-credential-interface-objects>
fn relevant_credential_interface_objects(
    _options: &CredentialRequestOptions<DomTypeHolder>,
) -> Vec<CredentialInterfaceObject> {
    // Step 1. Let settings be the current settings object.
    // Step 2. Let relevant interface objects be an empty set.
    // Step 3. For each optionKey -> optionValue of options:
    // Step 3.1. Let credentialInterfaceObject be the Appropriate Interface Object (on settings’ global object) whose Options Member Identifier is optionKey.
    // Step 3.2. Assert: credentialInterfaceObject’s [[type]] slot equals the Credential Type whose Options Member Identifier is optionKey.
    // Step 3.3. Append credentialInterfaceObject to relevant interface objects.
    // See: https://www.w3.org/TR/credential-management-1/#sctn-cred-type-registry
    let relevant_interface_objects = vec![CredentialInterfaceObject::Password];
    // Step 3.4. Return relevant interface objects.
    relevant_interface_objects
}

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
    fn request_credential(&self, options: &CredentialRequestOptions<DomTypeHolder>) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        // Step 2. Assert: settings is a secure context.
        assert!(self.global().is_secure_context());
        // Step 3. Let document be settings’s relevant global object's associated Document.
        // Step 4. If document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !self.global().as_window().Document().is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        // Step 5. If options.signal is aborted, then return a promise rejected with options.signal’s abort reason.
        if options.signal.as_ref().map_or(false, |s| s.aborted()) {
            return Err(Error::Abort);
        }
        // Step 6. Let interfaces be options’s relevant credential interface objects.
        let interfaces = relevant_credential_interface_objects(options);
        // Step 7. If interfaces is empty, then return a promise rejected with a "NotSupportedError" DOMException.
        if interfaces.is_empty() {
            return Err(Error::NotSupported);
        }
        // TODO: Step 8.
        // Step 9. Let origin be settings’ origin.
        let origin = self.global().origin().immutable().clone();
        // TODO: Step 10. Let sameOriginWithAncestors be true if settings is same-origin with its ancestors, and false otherwise.
        // TODO: Step 11
        // Step 12. Let p be a new promise.
        let promise = Promise::new(&*self.global(), CanGc::note());
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_self = Trusted::new(self);
        // TODO: Step 13. Run the following steps in parallel
        // TODO: Step 14. React to p
        // Step 15. Return p.
        Ok(promise)
    }

    /// <https://www.w3.org/TR/credential-management-1/#abstract-opdef-store-a-credential>
    fn store_credential(&self, credential: &Credential) -> Fallible<Rc<Promise>> {
        // Step 1. Let settings be the current settings object.
        // Step 2. Assert: settings is a secure context.
        assert!(self.global().is_secure_context());
        // Step 3. If settings’s relevant global object's associated Document is not fully active, then return a promise rejected with an "InvalidStateError" DOMException.
        if !self.global().as_window().Document().is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        // TODO: Step 4. Let sameOriginWithAncestors be true if the current settings object is same-origin with its ancestors, and false otherwise.
        // Step 5. Let p be a new promise.
        let promise = Promise::new(&*self.global(), CanGc::note());
        // TODO: Step 6. If settings’ active credential types contains credential’s [[type]], return a promise rejected with a "NotAllowedError" DOMException.
        // TODO: Step 7. Append credential’s [[type]] to settings’ active credential types.
        // TODO: Step 8. Run the following steps in parallel
        // Step 9. React to p
        // Step 10. Return p.
        Ok(promise)
    }
}

impl CredentialsContainerMethods<DomTypeHolder> for CredentialsContainer {
    // https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-get
    fn Get(&self, options: &CredentialRequestOptions<DomTypeHolder>) -> Fallible<Rc<Promise>> {
        self.request_credential(options)
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-store
    fn Store(&self, credential: &Credential) -> Fallible<Rc<Promise>> {
        self.store_credential(credential)
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-create
    fn Create(&self, _options: &CredentialCreationOptions<DomTypeHolder>) -> Fallible<Rc<Promise>> {
        Err(Error::NotSupported)
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-preventsilentaccess
    fn PreventSilentAccess(&self) -> Fallible<Rc<Promise>> {
        Err(Error::NotSupported)
    }
}
