/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fmt;

use dom_struct::dom_struct;
use js::jsapi::CompilationType;
use js::rust::HandleValue;

use crate::dom::bindings::codegen::Bindings::TrustedScriptBinding::TrustedScriptMethods;
use crate::dom::bindings::codegen::UnionTypes::TrustedScriptOrString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::CspReporting;
use crate::dom::globalscope::GlobalScope;
use crate::dom::trustedtypepolicy::TrustedType;
use crate::dom::trustedtypepolicyfactory::{DEFAULT_SCRIPT_SINK_GROUP, TrustedTypePolicyFactory};
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub struct TrustedScript {
    reflector_: Reflector,

    data: DOMString,
}

impl TrustedScript {
    fn new_inherited(data: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            data,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(data: DOMString, global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(data)), global, can_gc)
    }

    pub(crate) fn get_trusted_script_compliant_string(
        global: &GlobalScope,
        value: TrustedScriptOrString,
        sink: &str,
        can_gc: CanGc,
    ) -> Fallible<DOMString> {
        match value {
            TrustedScriptOrString::String(value) => {
                TrustedTypePolicyFactory::get_trusted_type_compliant_string(
                    TrustedType::TrustedScript,
                    global,
                    value,
                    sink,
                    DEFAULT_SCRIPT_SINK_GROUP,
                    can_gc,
                )
            },

            TrustedScriptOrString::TrustedScript(trusted_script) => Ok(trusted_script.data.clone()),
        }
    }

    pub(crate) fn data(&self) -> &DOMString {
        &self.data
    }

    /// <https://www.w3.org/TR/CSP/#can-compile-strings>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn can_compile_string_with_trusted_type(
        cx: JSContext,
        global: &GlobalScope,
        code_string: DOMString,
        compilation_type: CompilationType,
        parameter_strings: Vec<DOMString>,
        body_string: DOMString,
        parameter_args: Vec<TrustedScriptOrString>,
        body_arg: HandleValue,
        can_gc: CanGc,
    ) -> bool {
        // Step 2.1. Let compilationSink be "Function" if compilationType is "FUNCTION",
        // and "eval" otherwise.
        let compilation_sink = if compilation_type == CompilationType::Function {
            "Function"
        } else {
            "eval"
        };
        // Step 2.2. Let isTrusted be true if bodyArg implements TrustedScript,
        // and false otherwise.
        let mut is_trusted = match TrustedTypePolicyFactory::is_trusted_script(cx, body_arg) {
            // Step 2.3. If isTrusted is true then:
            Ok(trusted_script) => {
                // Step 2.3.1. If bodyString is not equal to bodyArg’s data, set isTrusted to false.
                body_string == trusted_script.data
            },
            _ => false,
        };
        // Step 2.4. If isTrusted is true, then:
        if is_trusted {
            // Step 2.4.1. Assert: parameterArgs’ [list/size=] is equal to [parameterStrings]' size.
            assert!(parameter_args.len() == parameter_strings.len());
            // Step 2.4.2. For each index of the range 0 to |parameterArgs]' [list/size=]:
            for index in 0..parameter_args.len() {
                // Step 2.4.2.1. Let arg be parameterArgs[index].
                match &parameter_args[index] {
                    // Step 2.4.2.2. If arg implements TrustedScript, then:
                    TrustedScriptOrString::TrustedScript(trusted_script) => {
                        // Step 2.4.2.2.1. if parameterStrings[index] is not equal to arg’s data,
                        // set isTrusted to false.
                        if parameter_strings[index] != *trusted_script.data() {
                            is_trusted = false;
                        }
                    },
                    // Step 2.4.2.3. Otherwise, set isTrusted to false.
                    TrustedScriptOrString::String(_) => {
                        is_trusted = false;
                    },
                }
            }
        }
        // Step 2.5. Let sourceToValidate be a new TrustedScript object created in realm
        // whose data is set to codeString if isTrusted is true, and codeString otherwise.
        let source_string = if is_trusted {
            // We don't need to call the compliant string algorithm, as it would immediately
            // unroll the type as allowed by copying the data. This allows us to skip creating
            // the DOM object.
            code_string
        } else {
            // Step 2.6. Let sourceString be the result of executing the
            // Get Trusted Type compliant string algorithm, with TrustedScript, realm,
            // sourceToValidate, compilationSink, and 'script'.
            match TrustedScript::get_trusted_script_compliant_string(
                global,
                TrustedScriptOrString::String(code_string.clone()),
                compilation_sink,
                can_gc,
            ) {
                // Step 2.7. If the algorithm throws an error, throw an EvalError.
                Err(_) => {
                    return false;
                },
                Ok(source_string) => {
                    // Step 2.8. If sourceString is not equal to codeString, throw an EvalError.
                    if source_string != code_string {
                        return false;
                    }
                    source_string
                },
            }
        };
        global
            .get_csp_list()
            .is_js_evaluation_allowed(global, &source_string.str())
    }
}

impl fmt::Display for TrustedScript {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.data.str())
    }
}

impl TrustedScriptMethods<crate::DomTypeHolder> for TrustedScript {
    /// <https://www.w3.org/TR/trusted-types/#trustedscript-stringification-behavior>
    fn Stringifier(&self) -> DOMString {
        self.data.clone()
    }

    /// <https://www.w3.org/TR/trusted-types/#dom-trustedscript-tojson>
    fn ToJSON(&self) -> DOMString {
        self.data.clone()
    }
}
