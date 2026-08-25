/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::str::{DOMString, USVString};

use crate::dom::bindings::codegen::Bindings::CredentialBinding::CredentialMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct Credential {
    reflector_: Reflector,
    id: USVString,
    credential_type: DOMString,
}

impl Credential {
    pub(crate) fn new_inherited(id: USVString, credential_type: DOMString) -> Credential {
        Credential {
            reflector_: Reflector::new(),
            id,
            credential_type,
        }
    }

    #[expect(dead_code)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        id: USVString,
        credential_type: DOMString,
    ) -> DomRoot<Credential> {
        reflect_dom_object_with_cx(
            Box::new(Credential::new_inherited(id, credential_type)),
            global,
            cx,
        )
    }
}

impl CredentialMethods<DomTypeHolder> for Credential {
    /// <https://www.w3.org/TR/credential-management-1/#dom-credential-id>
    fn Id(&self) -> USVString {
        self.id.clone()
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credential-type>
    fn Type(&self) -> DOMString {
        self.credential_type.clone()
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credential-isconditionalmediationavailable>
    fn IsConditionalMediationAvailable(cx: &mut CurrentRealm, _global: &Window) -> Rc<Promise> {
        Promise::new_in_realm(cx)
    }

    /// <https://www.w3.org/TR/credential-management-1/#dom-credential-willrequestconditionalcreation>
    fn WillRequestConditionalCreation(cx: &mut CurrentRealm, _global: &Window) -> Rc<Promise> {
        Promise::new_in_realm(cx)
    }
}
