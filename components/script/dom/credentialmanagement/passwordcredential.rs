/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::PasswordCredentialBinding::PasswordCredentialData;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use script_bindings::str::USVString;

use crate::dom::bindings::codegen::Bindings::PasswordCredentialBinding::PasswordCredentialMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::root::DomRoot;
use crate::dom::credentialmanagement::credential::Credential;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::window::Window;

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

    fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        id: USVString,
        origin: USVString,
        password: USVString,
    ) -> DomRoot<PasswordCredential> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(PasswordCredential::new_inherited(id, origin, password)),
            global,
            proto,
            cx,
        )
    }
}

impl PasswordCredentialMethods<DomTypeHolder> for PasswordCredential {
    fn Password(&self) -> USVString {
        self.password.clone()
    }

    fn Constructor(
        _cx: &mut JSContext,
        _global: &Window,
        _proto: Option<HandleObject>,
        _form: &HTMLFormElement,
    ) -> Fallible<DomRoot<PasswordCredential>> {
        Err(Error::NotSupported(None))
    }

    fn Constructor_(
        cx: &mut JSContext,
        global: &Window,
        proto: Option<HandleObject>,
        data: &PasswordCredentialData,
    ) -> Fallible<DomRoot<PasswordCredential>> {
        Ok(Self::new_with_proto(
            cx,
            global.as_global_scope(),
            proto,
            data.parent.id.clone(),
            data.origin.clone(),
            data.password.clone(),
        ))
    }
}
