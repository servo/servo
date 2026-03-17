/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use dom_struct::dom_struct;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::TrustedHTMLBinding::TrustedHTMLMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    TrustedHTMLOrNullIsEmptyString, TrustedHTMLOrString,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::trustedtypes::trustedtypepolicy::TrustedType;
use crate::dom::trustedtypes::trustedtypepolicyfactory::{
    DEFAULT_SCRIPT_SINK_GROUP, TrustedTypePolicyFactory,
};

#[dom_struct]
pub struct TrustedHTML {
    reflector_: Reflector,

    data: DOMString,
}

impl TrustedHTML {
    fn new_inherited(data: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            data,
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        data: DOMString,
        global: &GlobalScope,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited(data)), global, cx)
    }

    pub(crate) fn get_trusted_type_compliant_string(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        value: TrustedHTMLOrString,
        sink: &str,
    ) -> Fallible<DOMString> {
        match value {
            TrustedHTMLOrString::String(value) => {
                TrustedTypePolicyFactory::get_trusted_type_compliant_string(
                    cx,
                    TrustedType::TrustedHTML,
                    global,
                    value,
                    sink,
                    DEFAULT_SCRIPT_SINK_GROUP,
                )
            },

            TrustedHTMLOrString::TrustedHTML(trusted_html) => Ok(trusted_html.data.clone()),
        }
    }

    pub(crate) fn data(&self) -> &DOMString {
        &self.data
    }
}

impl fmt::Display for TrustedHTML {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.data.str())
    }
}

impl TrustedHTMLMethods<crate::DomTypeHolder> for TrustedHTML {
    /// <https://www.w3.org/TR/trusted-types/#trustedhtml-stringification-behavior>
    fn Stringifier(&self) -> DOMString {
        self.data.clone()
    }

    /// <https://www.w3.org/TR/trusted-types/#dom-trustedhtml-tojson>
    fn ToJSON(&self) -> DOMString {
        self.data.clone()
    }
}

impl Convert<TrustedHTMLOrString> for TrustedHTMLOrNullIsEmptyString {
    fn convert(self) -> TrustedHTMLOrString {
        match self {
            TrustedHTMLOrNullIsEmptyString::TrustedHTML(trusted_html) => {
                TrustedHTMLOrString::TrustedHTML(trusted_html)
            },
            TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(str) => {
                TrustedHTMLOrString::String(str)
            },
        }
    }
}
