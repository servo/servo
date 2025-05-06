/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TrustedHTMLBinding::TrustedHTMLMethods;
use crate::dom::bindings::codegen::UnionTypes::TrustedHTMLOrString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::trustedtypepolicy::TrustedType;
use crate::dom::trustedtypepolicyfactory::TrustedTypePolicyFactory;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct TrustedHTML {
    reflector_: Reflector,

    data: String,
}

impl TrustedHTML {
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

    pub(crate) fn get_trusted_script_compliant_string(
        global: &GlobalScope,
        value: TrustedHTMLOrString,
        containing_class: &str,
        field: &str,
        can_gc: CanGc,
    ) -> Fallible<String> {
        match value {
            TrustedHTMLOrString::String(value) => {
                let sink = format!("{} {}", containing_class, field);
                TrustedTypePolicyFactory::get_trusted_type_compliant_string(
                    TrustedType::TrustedHTML,
                    global,
                    value.as_ref().to_owned(),
                    &sink,
                    "'script'",
                    can_gc,
                )
            },

            TrustedHTMLOrString::TrustedHTML(trusted_html) => Ok(trusted_html.to_string()),
        }
    }
}

impl fmt::Display for TrustedHTML {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.data)
    }
}

impl TrustedHTMLMethods<crate::DomTypeHolder> for TrustedHTML {
    /// <https://www.w3.org/TR/trusted-types/#trustedhtml-stringification-behavior>
    fn Stringifier(&self) -> DOMString {
        DOMString::from(&*self.data)
    }

    /// <https://www.w3.org/TR/trusted-types/#dom-trustedhtml-tojson>
    fn ToJSON(&self) -> DOMString {
        DOMString::from(&*self.data)
    }
}
