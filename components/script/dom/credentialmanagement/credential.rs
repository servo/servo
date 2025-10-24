/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::rc::Rc;

use dom_struct::dom_struct;
use script_bindings::realms::{AlreadyInRealm, InRealm};
use script_bindings::str::{DOMString, USVString};

use crate::dom::bindings::codegen::Bindings::CredentialBinding::CredentialMethods;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

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
        global: &GlobalScope,
        id: USVString,
        credential_type: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<Credential> {
        reflect_dom_object(
            Box::new(Credential::new_inherited(id, credential_type)),
            global,
            can_gc,
        )
    }
}

impl CredentialMethods<DomTypeHolder> for Credential {
    // https://www.w3.org/TR/credential-management-1/#dom-credential-id
    fn Id(&self) -> USVString {
        self.id.clone()
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credential-type
    fn Type(&self) -> DOMString {
        self.credential_type.clone()
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credential-isconditionalmediationavailable
    fn IsConditionalMediationAvailable(_global: &Window, can_gc: CanGc) -> Rc<Promise> {
        let in_realm_proof = AlreadyInRealm::assert::<DomTypeHolder>();
        // FIXME:(arihant2math) return false
        Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc)
    }

    // https://www.w3.org/TR/credential-management-1/#dom-credential-willrequestconditionalcreation
    fn WillRequestConditionalCreation(_global: &Window, can_gc: CanGc) -> Rc<Promise> {
        let in_realm_proof = AlreadyInRealm::assert::<DomTypeHolder>();
        Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc)
    }
}
