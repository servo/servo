/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::HandleValue;
use strum_macros::IntoStaticStr;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::TrustedTypePolicyBinding::TrustedTypePolicyMethods;
use crate::dom::bindings::codegen::Bindings::TrustedTypePolicyFactoryBinding::{
    CreateHTMLCallback, CreateScriptCallback, CreateScriptURLCallback, TrustedTypePolicyOptions,
};
use crate::dom::bindings::error::Error::Type;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{DomGlobal, DomObject, Reflector, reflect_dom_object};
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

    #[ignore_malloc_size_of = "Rc has unclear ownership"]
    create_html: Option<Rc<CreateHTMLCallback>>,
    #[ignore_malloc_size_of = "Rc has unclear ownership"]
    create_script: Option<Rc<CreateScriptCallback>>,
    #[ignore_malloc_size_of = "Rc has unclear ownership"]
    create_script_url: Option<Rc<CreateScriptURLCallback>>,
}

#[derive(Clone, IntoStaticStr)]
pub(crate) enum TrustedType {
    TrustedHTML,
    TrustedScript,
    TrustedScriptURL,
}

impl TrustedTypePolicy {
    fn new_inherited(name: String, options: &TrustedTypePolicyOptions) -> Self {
        Self {
            reflector_: Reflector::new(),
            name,
            create_html: options.createHTML.clone(),
            create_script: options.createScript.clone(),
            create_script_url: options.createScriptURL.clone(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        name: String,
        options: &TrustedTypePolicyOptions,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(name, options)), global, can_gc)
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-policy-value-algorithm>
    fn check_callback_if_missing(throw_if_missing: bool) -> Fallible<Option<String>> {
        // Step 3.1: If throwIfMissing throw a TypeError.
        if throw_if_missing {
            Err(Type("Cannot find type".to_owned()))
        } else {
            // Step 3.2: Else return null.
            Ok(None)
        }
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-policy-value-algorithm>
    pub(crate) fn get_trusted_type_policy_value(
        &self,
        expected_type: TrustedType,
        cx: JSContext,
        input: DOMString,
        arguments: Vec<HandleValue>,
        throw_if_missing: bool,
        can_gc: CanGc,
    ) -> Fallible<Option<String>> {
        rooted!(in(*cx) let this_object: *mut JSObject);
        // Step 1: Let functionName be a function name for the given trustedTypeName, based on the following table:
        match expected_type {
            TrustedType::TrustedHTML => match &self.create_html {
                // Step 3: If function is null, then:
                None => TrustedTypePolicy::check_callback_if_missing(throw_if_missing),
                // Step 2: Let function be policy’s options[functionName].
                Some(callback) => {
                    // Step 4: Let policyValue be the result of invoking function with value as a first argument,
                    // items of arguments as subsequent arguments, and callback **this** value set to null,
                    // rethrowing any exceptions.
                    callback
                        .Call_(
                            &this_object.handle(),
                            input,
                            arguments,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                        .map(|result| result.map(|str| str.as_ref().to_owned()))
                },
            },
            TrustedType::TrustedScript => match &self.create_script {
                // Step 3: If function is null, then:
                None => TrustedTypePolicy::check_callback_if_missing(throw_if_missing),
                // Step 2: Let function be policy’s options[functionName].
                Some(callback) => {
                    // Step 4: Let policyValue be the result of invoking function with value as a first argument,
                    // items of arguments as subsequent arguments, and callback **this** value set to null,
                    // rethrowing any exceptions.
                    callback
                        .Call_(
                            &this_object.handle(),
                            input,
                            arguments,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                        .map(|result| result.map(|str| str.as_ref().to_owned()))
                },
            },
            TrustedType::TrustedScriptURL => match &self.create_script_url {
                // Step 3: If function is null, then:
                None => TrustedTypePolicy::check_callback_if_missing(throw_if_missing),
                // Step 2: Let function be policy’s options[functionName].
                Some(callback) => {
                    // Step 4: Let policyValue be the result of invoking function with value as a first argument,
                    // items of arguments as subsequent arguments, and callback **this** value set to null,
                    // rethrowing any exceptions.
                    callback
                        .Call_(
                            &this_object.handle(),
                            input,
                            arguments,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                        .map(|result| result.map(|str| str.as_ref().to_owned()))
                },
            },
        }
    }

    /// This does not take all arguments as specified. That's because the return type of the
    /// trusted type function and object are not the same. 2 of the 3 string callbacks return
    /// a DOMString, while the other one returns an USVString. Additionally, all three callbacks
    /// have a unique type signature in WebIDL.
    ///
    /// To circumvent these type problems, rather than implementing the full functionality here,
    /// part of the algorithm is implemented on the caller side. There, we only call the callback
    /// and create the object. The rest of the machinery is ensuring the right values pass through
    /// to the relevant callbacks.
    ///
    /// <https://w3c.github.io/trusted-types/dist/spec/#create-a-trusted-type-algorithm>
    pub(crate) fn create_trusted_type<R, TrustedTypeCallback>(
        &self,
        expected_type: TrustedType,
        cx: JSContext,
        input: DOMString,
        arguments: Vec<HandleValue>,
        trusted_type_creation_callback: TrustedTypeCallback,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<R>>
    where
        R: DomObject,
        TrustedTypeCallback: FnOnce(String) -> DomRoot<R>,
    {
        // Step 1: Let policyValue be the result of executing Get Trusted Type policy value
        // with the same arguments as this algorithm and additionally true as throwIfMissing.
        let policy_value =
            self.get_trusted_type_policy_value(expected_type, cx, input, arguments, true, can_gc);
        match policy_value {
            // Step 2: If the algorithm threw an error, rethrow the error and abort the following steps.
            Err(error) => Err(error),
            Ok(policy_value) => {
                // Step 3: Let dataString be the result of stringifying policyValue.
                let data_string = match policy_value {
                    Some(value) => value,
                    // Step 4: If policyValue is null or undefined, set dataString to the empty string.
                    None => "".to_owned(),
                };
                // Step 5: Return a new instance of an interface with a type name trustedTypeName,
                // with its associated data value set to dataString.
                Ok(trusted_type_creation_callback(data_string))
            },
        }
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
        cx: JSContext,
        input: DOMString,
        arguments: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TrustedHTML>> {
        self.create_trusted_type(
            TrustedType::TrustedHTML,
            cx,
            input,
            arguments,
            |data_string| TrustedHTML::new(data_string, &self.global(), can_gc),
            can_gc,
        )
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicy-createscript>
    fn CreateScript(
        &self,
        cx: JSContext,
        input: DOMString,
        arguments: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TrustedScript>> {
        self.create_trusted_type(
            TrustedType::TrustedScript,
            cx,
            input,
            arguments,
            |data_string| TrustedScript::new(data_string, &self.global(), can_gc),
            can_gc,
        )
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicy-createscripturl>
    fn CreateScriptURL(
        &self,
        cx: JSContext,
        input: DOMString,
        arguments: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TrustedScriptURL>> {
        self.create_trusted_type(
            TrustedType::TrustedScriptURL,
            cx,
            input,
            arguments,
            |data_string| TrustedScriptURL::new(data_string, &self.global(), can_gc),
            can_gc,
        )
    }
}
