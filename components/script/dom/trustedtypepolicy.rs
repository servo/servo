/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleValue;

use crate::dom::bindings::codegen::Bindings::TrustedTypePolicyBinding::TrustedTypePolicyMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::trustedhtml::TrustedHTML;
use crate::dom::trustedscript::TrustedScript;
use crate::dom::trustedscripturl::TrustedScriptURL;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub struct TrustedTypePolicy {
    reflector_: Reflector,

    name: String,
}

impl TrustedTypePolicy {
    fn new_inherited(name: String) -> Self {
        Self {
            reflector_: Reflector::new(),
            name,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(name: String, global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(name)), global, can_gc)
    }
}

impl TrustedTypePolicyMethods<crate::DomTypeHolder> for TrustedTypePolicy {
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicy-name>
    fn Name(&self) -> DOMString {
        DOMString::from(&*self.name)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicy-createhtml>
    fn CreateHTML(
        &self,
        _: JSContext,
        data: DOMString,
        _: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> DomRoot<TrustedHTML> {
        // TODO(36258): handle arguments
        TrustedHTML::new(data.to_string(), &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicy-createscript>
    fn CreateScript(
        &self,
        _: JSContext,
        data: DOMString,
        _: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> DomRoot<TrustedScript> {
        // TODO(36258): handle arguments
        TrustedScript::new(data.to_string(), &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicy-createscripturl>
    fn CreateScriptURL(
        &self,
        _: JSContext,
        data: DOMString,
        _: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> DomRoot<TrustedScriptURL> {
        // TODO(36258): handle arguments
        TrustedScriptURL::new(data.to_string(), &self.global(), can_gc)
    }
}
