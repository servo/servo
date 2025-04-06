/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TrustedScriptBinding::TrustedScriptMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct TrustedScript {
    reflector_: Reflector,

    data: String,
}

impl TrustedScript {
    fn new_inherited(data: String) -> Self {
        Self {
            reflector_: Reflector::new(),
            data,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(data: String, global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(data)), global, can_gc)
    }
}

impl TrustedScriptMethods<crate::DomTypeHolder> for TrustedScript {
    /// <https://www.w3.org/TR/trusted-types/#trustedscript-stringification-behavior>
    fn Stringifier(&self) -> DOMString {
        DOMString::from(&*self.data)
    }

    /// <https://www.w3.org/TR/trusted-types/#dom-trustedscript-tojson>
    fn ToJSON(&self) -> DOMString {
        DOMString::from(&*self.data)
    }
}
