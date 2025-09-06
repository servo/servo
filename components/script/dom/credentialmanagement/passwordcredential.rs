/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::PasswordCredentialBinding::PasswordCredentialData;
use script_bindings::error::{Error, Fallible};
use script_bindings::str::USVString;

use crate::dom::bindings::codegen::Bindings::PasswordCredentialBinding::PasswordCredentialMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::{reflect_dom_object, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::credentialmanagement::credential::Credential;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PasswordCredential {
    credential: Credential,
    origin: USVString,
    password: USVString,
}

impl PasswordCredential {
    fn new_inherited(id: USVString, origin: USVString, password: USVString) -> PasswordCredential {
        PasswordCredential {
            credential: Credential::new_inherited(id, "password".into()),
            origin,
            password,
        }
    }

    #[expect(dead_code)]
    pub(crate) fn new(
        global: &GlobalScope,
        id: USVString,
        origin: USVString,
        password: USVString,
        can_gc: CanGc,
    ) -> DomRoot<PasswordCredential> {
        reflect_dom_object(
            Box::new(PasswordCredential::new_inherited(id, origin, password)),
            global,
            can_gc,
        )
    }

    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        id: USVString,
        origin: USVString,
        password: USVString,
        can_gc: CanGc,
    ) -> DomRoot<PasswordCredential> {
        reflect_dom_object_with_proto(
            Box::new(PasswordCredential::new_inherited(id, origin, password)),
            global,
            proto,
            can_gc,
        )
    }
}

impl PasswordCredentialMethods<DomTypeHolder> for PasswordCredential {
    fn Password(&self) -> USVString {
        self.password.clone()
    }

    fn Constructor(
        _global: &Window,
        _proto: Option<HandleObject>,
        _can_gc: CanGc,
        _form: &HTMLFormElement,
    ) -> Fallible<DomRoot<PasswordCredential>> {
        Err(Error::NotSupported)
    }

    fn Constructor_(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        data: &PasswordCredentialData,
    ) -> Fallible<DomRoot<PasswordCredential>> {
        Ok(Self::new_with_proto(
            global.as_global_scope(),
            proto,
            data.parent.id.clone(),
            data.origin.clone(),
            data.password.clone(),
            can_gc,
        ))
    }
}
