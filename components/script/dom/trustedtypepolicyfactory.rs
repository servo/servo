/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;

use dom_struct::dom_struct;
use js::rust::HandleValue;

use crate::dom::bindings::codegen::Bindings::TrustedTypePolicyFactoryBinding::{
    TrustedTypePolicyFactoryMethods, TrustedTypePolicyOptions,
};
use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::trustedhtml::TrustedHTML;
use crate::dom::trustedscript::TrustedScript;
use crate::dom::trustedscripturl::TrustedScriptURL;
use crate::dom::trustedtypepolicy::TrustedTypePolicy;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub struct TrustedTypePolicyFactory {
    reflector_: Reflector,

    default_policy: MutNullableDom<TrustedTypePolicy>,
    policy_names: RefCell<Vec<String>>,
}

impl TrustedTypePolicyFactory {
    fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
            default_policy: Default::default(),
            policy_names: RefCell::new(vec![]),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), global, can_gc)
    }

    /// <https://www.w3.org/TR/trusted-types/#create-trusted-type-policy-algorithm>
    fn create_trusted_type_policy(
        &self,
        policy_name: String,
        _options: &TrustedTypePolicyOptions,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TrustedTypePolicy>> {
        // TODO(36258): implement proper CSP check
        // Step 1: Let allowedByCSP be the result of executing Should Trusted Type policy creation be blocked by
        // Content Security Policy? algorithm with global, policyName and factory’s created policy names value.
        let allowed_by_csp = true;

        // Step 2: If allowedByCSP is "Blocked", throw a TypeError and abort further steps.
        if !allowed_by_csp {
            return Err(Error::Type("Not allowed by CSP".to_string()));
        }

        // Step 3: If policyName is default and the factory’s default policy value is not null, throw a TypeError
        // and abort further steps.
        if policy_name == "default" && self.default_policy.get().is_some() {
            return Err(Error::Type(
                "Already set default policy for factory".to_string(),
            ));
        }

        // Step 4: Let policy be a new TrustedTypePolicy object.
        // Step 5: Set policy’s name property value to policyName.
        let policy = TrustedTypePolicy::new(policy_name.clone(), global, can_gc);
        // Step 6: Set policy’s options value to «[ "createHTML" ->
        // options["createHTML", "createScript" -> options["createScript",
        // "createScriptURL" -> options["createScriptURL" ]».
        // TODO(36258): implement step 6
        // Step 7: If the policyName is default, set the factory’s default policy value to policy.
        if policy_name == "default" {
            self.default_policy.set(Some(&policy))
        }
        // Step 8: Append policyName to factory’s created policy names.
        self.policy_names.borrow_mut().push(policy_name);
        // Step 9: Return policy.
        Ok(policy)
    }
}

impl TrustedTypePolicyFactoryMethods<crate::DomTypeHolder> for TrustedTypePolicyFactory {
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-createpolicy>
    fn CreatePolicy(
        &self,
        policy_name: DOMString,
        options: &TrustedTypePolicyOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TrustedTypePolicy>> {
        self.create_trusted_type_policy(policy_name.to_string(), options, &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-ishtml>
    #[allow(unsafe_code)]
    fn IsHTML(&self, cx: JSContext, value: HandleValue) -> bool {
        if !value.get().is_object() {
            return false;
        }
        rooted!(in(*cx) let object = value.to_object());
        unsafe { root_from_object::<TrustedHTML>(object.get(), *cx).is_ok() }
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-isscript>
    #[allow(unsafe_code)]
    fn IsScript(&self, cx: JSContext, value: HandleValue) -> bool {
        if !value.get().is_object() {
            return false;
        }
        rooted!(in(*cx) let object = value.to_object());
        unsafe { root_from_object::<TrustedScript>(object.get(), *cx).is_ok() }
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-isscripturl>
    #[allow(unsafe_code)]
    fn IsScriptURL(&self, cx: JSContext, value: HandleValue) -> bool {
        if !value.get().is_object() {
            return false;
        }
        rooted!(in(*cx) let object = value.to_object());
        unsafe { root_from_object::<TrustedScriptURL>(object.get(), *cx).is_ok() }
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-emptyhtml>
    fn EmptyHTML(&self, can_gc: CanGc) -> DomRoot<TrustedHTML> {
        TrustedHTML::new("".to_string(), &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-emptyscript>
    fn EmptyScript(&self, can_gc: CanGc) -> DomRoot<TrustedScript> {
        TrustedScript::new("".to_string(), &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-getattributetype>
    fn GetAttributeType(
        &self,
        _: DOMString,
        _: DOMString,
        _: Option<DOMString>,
        _: Option<DOMString>,
    ) -> Option<DOMString> {
        // TODO(36258): implement algorithm
        Some(DOMString::from("".to_string()))
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-getpropertytype>
    fn GetPropertyType(
        &self,
        _: DOMString,
        _: DOMString,
        _: Option<DOMString>,
    ) -> Option<DOMString> {
        // TODO(36258): implement algorithm
        Some(DOMString::from("".to_string()))
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-defaultpolicy>
    fn GetDefaultPolicy(&self) -> Option<DomRoot<TrustedTypePolicy>> {
        self.default_policy.get()
    }
}
