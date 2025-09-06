/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::rc::Rc;

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::CredentialsContainerBinding::{
    CredentialCreationOptions, CredentialRequestOptions,
};
use script_bindings::error::{Error, Fallible};

use crate::dom::bindings::codegen::Bindings::CredentialsContainerBinding::CredentialsContainerMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
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
}

impl CredentialsContainerMethods<DomTypeHolder> for CredentialsContainer {
    // https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-get
    fn Get(&self, _options: &CredentialRequestOptions<DomTypeHolder>) -> Fallible<Rc<Promise>> {
        Err(Error::NotSupported)
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credentialscontainer-store
    fn Store(&self, _credential: &Credential) -> Fallible<Rc<Promise>> {
        Err(Error::NotSupported)
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
