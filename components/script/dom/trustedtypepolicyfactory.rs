/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;

use dom_struct::dom_struct;
use html5ever::{LocalName, Namespace, QualName, local_name, ns};
use js::jsval::NullValue;
use js::rust::HandleValue;
use script_bindings::conversions::SafeToJSValConvertible;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::TrustedTypePolicyFactoryBinding::{
    TrustedTypePolicyFactoryMethods, TrustedTypePolicyOptions,
};
use crate::dom::bindings::codegen::UnionTypes::TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString as TrustedTypeOrString;
use crate::dom::bindings::conversions::root_from_handlevalue;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::CspReporting;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::trustedhtml::TrustedHTML;
use crate::dom::trustedscript::TrustedScript;
use crate::dom::trustedscripturl::TrustedScriptURL;
use crate::dom::trustedtypepolicy::{TrustedType, TrustedTypePolicy};
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub struct TrustedTypePolicyFactory {
    reflector_: Reflector,

    default_policy: MutNullableDom<TrustedTypePolicy>,
    policy_names: RefCell<Vec<String>>,
}

pub(crate) static DEFAULT_SCRIPT_SINK_GROUP: &str = "'script'";

impl Convert<DOMString> for TrustedTypeOrString {
    fn convert(self) -> DOMString {
        match self {
            TrustedTypeOrString::TrustedHTML(trusted_html) => trusted_html.data(),
            TrustedTypeOrString::TrustedScript(trusted_script) => trusted_script.data(),
            TrustedTypeOrString::TrustedScriptURL(trusted_script_url) => trusted_script_url.data(),
            TrustedTypeOrString::String(str_) => str_,
        }
    }
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
        options: &TrustedTypePolicyOptions,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TrustedTypePolicy>> {
        // Step 1: Let allowedByCSP be the result of executing Should Trusted Type policy creation be blocked by
        // Content Security Policy? algorithm with global, policyName and factory’s created policy names value.
        let allowed_by_csp = global
            .get_csp_list()
            .is_trusted_type_policy_creation_allowed(
                global,
                policy_name.clone(),
                self.policy_names.borrow().clone(),
            );

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
        // Step 6: Set policy’s options value to «[ "createHTML" ->
        // options["createHTML", "createScript" -> options["createScript",
        // "createScriptURL" -> options["createScriptURL" ]».
        let policy = TrustedTypePolicy::new(policy_name.clone(), options, global, can_gc);
        // Step 7: If the policyName is default, set the factory’s default policy value to policy.
        if policy_name == "default" {
            self.default_policy.set(Some(&policy))
        }
        // Step 8: Append policyName to factory’s created policy names.
        self.policy_names.borrow_mut().push(policy_name);
        // Step 9: Return policy.
        Ok(policy)
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#abstract-opdef-get-trusted-type-data-for-attribute>
    #[allow(clippy::if_same_then_else)]
    fn get_trusted_type_data_for_attribute(
        element_namespace: &Namespace,
        element_name: &LocalName,
        attribute: &str,
        attribute_namespace: Option<&Namespace>,
    ) -> Option<(TrustedType, String)> {
        // Step 1: Let data be null.
        //
        // We return the if directly
        // Step 2: If attributeNs is null, « HTML namespace, SVG namespace, MathML namespace » contains
        // element’s namespace, and attribute is the name of an event handler content attribute:
        if attribute_namespace.is_none() &&
            matches!(*element_namespace, ns!(html) | ns!(svg) | ns!(mathml)) &&
            EventTarget::is_content_event_handler(attribute)
        {
            // Step 2.1. Return (Element, null, attribute, TrustedScript, "Element " + attribute).
            return Some((
                TrustedType::TrustedScript,
                "Element ".to_owned() + attribute,
            ));
        }
        // Step 3: Find the row in the following table, where element is in the first column,
        // attributeNs is in the second column, and attribute is in the third column.
        // If a matching row is found, set data to that row.
        // Step 4: Return data.
        if *element_namespace == ns!(html) &&
            *element_name == local_name!("iframe") &&
            attribute_namespace.is_none() &&
            attribute == "srcdoc"
        {
            Some((
                TrustedType::TrustedHTML,
                "HTMLIFrameElement srcdoc".to_owned(),
            ))
        } else if *element_namespace == ns!(html) &&
            *element_name == local_name!("script") &&
            attribute_namespace.is_none() &&
            attribute == "src"
        {
            Some((
                TrustedType::TrustedScriptURL,
                "HTMLScriptElement src".to_owned(),
            ))
        } else if *element_namespace == ns!(svg) &&
            *element_name == local_name!("script") &&
            attribute_namespace.is_none() &&
            attribute == "href"
        {
            Some((
                TrustedType::TrustedScriptURL,
                "SVGScriptElement href".to_owned(),
            ))
        } else if *element_namespace == ns!(svg) &&
            *element_name == local_name!("script") &&
            attribute_namespace == Some(&ns!(xlink)) &&
            attribute == "href"
        {
            Some((
                TrustedType::TrustedScriptURL,
                "SVGScriptElement href".to_owned(),
            ))
        } else {
            None
        }
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#validate-attribute-mutation>
    pub(crate) fn get_trusted_types_compliant_attribute_value(
        element_namespace: &Namespace,
        element_name: &LocalName,
        attribute: &str,
        attribute_namespace: Option<&Namespace>,
        new_value: TrustedTypeOrString,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<DOMString> {
        // Step 1. If attributeNs is the empty string, set attributeNs to null.
        let attribute_namespace =
            attribute_namespace.and_then(|a| if *a == ns!() { None } else { Some(a) });
        // Step 2. Set attributeData to the result of Get Trusted Type data for attribute algorithm,
        // with the following arguments:
        let Some(attribute_data) = Self::get_trusted_type_data_for_attribute(
            element_namespace,
            element_name,
            attribute,
            attribute_namespace,
        ) else {
            // Step 3. If attributeData is null, then:
            // Step 3.1. If newValue is a string, return newValue.
            // Step 3.2. Assert: newValue is TrustedHTML or TrustedScript or TrustedScriptURL.
            // Step 3.3. Return value’s associated data.
            return Ok(new_value.convert());
        };
        // Step 4. Let expectedType be the value of the fourth member of attributeData.
        // Step 5. Let sink be the value of the fifth member of attributeData.
        let (expected_type, sink) = attribute_data;
        let new_value = if let TrustedTypeOrString::String(str_) = new_value {
            str_
        } else {
            // If the type was already trusted, we should return immediately as
            // all callers of `get_trusted_type_compliant_string` implement this
            // check themselves. However, we should only do this if it matches
            // the expected type.
            if expected_type.matches_idl_trusted_type(&new_value) {
                return Ok(new_value.convert());
            }
            new_value.convert()
        };
        // Step 6. Return the result of executing Get Trusted Type compliant string with the following arguments:
        // If the algorithm threw an error, rethrow the error.
        Self::get_trusted_type_compliant_string(
            expected_type,
            global,
            new_value,
            &sink,
            DEFAULT_SCRIPT_SINK_GROUP,
            can_gc,
        )
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#process-value-with-a-default-policy-algorithm>
    pub(crate) fn process_value_with_default_policy(
        expected_type: TrustedType,
        global: &GlobalScope,
        input: DOMString,
        sink: &str,
        can_gc: CanGc,
    ) -> Fallible<Option<DOMString>> {
        // Step 1: Let defaultPolicy be the value of global’s trusted type policy factory's default policy.
        let global_policy_factory = global.trusted_types(can_gc);
        let default_policy = match global_policy_factory.default_policy.get() {
            None => return Ok(None),
            Some(default_policy) => default_policy,
        };
        let cx = GlobalScope::get_cx();
        // Step 2: Let policyValue be the result of executing Get Trusted Type policy value,
        // with the following arguments:
        rooted!(in(*cx) let mut trusted_type_name_value = NullValue());
        expected_type
            .clone()
            .as_ref()
            .safe_to_jsval(cx, trusted_type_name_value.handle_mut());

        rooted!(in(*cx) let mut sink_value = NullValue());
        sink.safe_to_jsval(cx, sink_value.handle_mut());

        let arguments = vec![trusted_type_name_value.handle(), sink_value.handle()];
        let policy_value = default_policy.get_trusted_type_policy_value(
            expected_type,
            input,
            arguments,
            false,
            can_gc,
        );
        let data_string = match policy_value {
            // Step 3: If the algorithm threw an error, rethrow the error and abort the following steps.
            Err(error) => return Err(error),
            Ok(policy_value) => match policy_value {
                // Step 4: If policyValue is null or undefined, return policyValue.
                None => return Ok(None),
                // Step 5: Let dataString be the result of stringifying policyValue.
                Some(policy_value) => policy_value,
            },
        };
        Ok(Some(data_string))
    }
    /// Step 1 is implemented by the caller
    /// <https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-string-algorithm>
    pub(crate) fn get_trusted_type_compliant_string(
        expected_type: TrustedType,
        global: &GlobalScope,
        input: DOMString,
        sink: &str,
        sink_group: &str,
        can_gc: CanGc,
    ) -> Fallible<DOMString> {
        // Step 2: Let requireTrustedTypes be the result of executing Does sink type require trusted types?
        // algorithm, passing global, sinkGroup, and true.
        let require_trusted_types = global
            .get_csp_list()
            .does_sink_type_require_trusted_types(sink_group, true);
        // Step 3: If requireTrustedTypes is false, return stringified input and abort these steps.
        if !require_trusted_types {
            return Ok(input);
        }
        // Step 4: Let convertedInput be the result of executing Process value with a default policy
        // with the same arguments as this algorithm.
        let converted_input = TrustedTypePolicyFactory::process_value_with_default_policy(
            expected_type,
            global,
            input.clone(),
            sink,
            can_gc,
        );
        // Step 5: If the algorithm threw an error, rethrow the error and abort the following steps.
        match converted_input? {
            // Step 6: If convertedInput is null or undefined, execute the following steps:
            None => {
                // Step 6.1: Let disposition be the result of executing Should sink type mismatch violation
                // be blocked by Content Security Policy? algorithm, passing global,
                // stringified input as source, sinkGroup and sink.
                let is_blocked = global
                    .get_csp_list()
                    .should_sink_type_mismatch_violation_be_blocked_by_csp(
                        global, sink, sink_group, &input,
                    );
                // Step 6.2: If disposition is “Allowed”, return stringified input and abort further steps.
                if !is_blocked {
                    Ok(input)
                } else {
                    // Step 6.3: Throw a TypeError and abort further steps.
                    Err(Error::Type(
                        "Cannot set value, expected trusted type".to_owned(),
                    ))
                }
            },
            // Step 8: Return stringified convertedInput.
            Some(converted_input) => Ok(converted_input),
        }
        // Step 7: Assert: convertedInput is an instance of expectedType.
        // TODO(https://github.com/w3c/trusted-types/issues/566): Implement when spec is resolved
    }

    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-isscript>
    #[allow(unsafe_code)]
    pub(crate) fn is_trusted_script(
        cx: JSContext,
        value: HandleValue,
    ) -> Result<DomRoot<TrustedScript>, ()> {
        unsafe { root_from_handlevalue::<TrustedScript>(value, *cx) }
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
        unsafe { root_from_handlevalue::<TrustedHTML>(value, *cx).is_ok() }
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-isscript>
    #[allow(unsafe_code)]
    fn IsScript(&self, cx: JSContext, value: HandleValue) -> bool {
        TrustedTypePolicyFactory::is_trusted_script(cx, value).is_ok()
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-isscripturl>
    #[allow(unsafe_code)]
    fn IsScriptURL(&self, cx: JSContext, value: HandleValue) -> bool {
        unsafe { root_from_handlevalue::<TrustedScriptURL>(value, *cx).is_ok() }
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-emptyhtml>
    fn EmptyHTML(&self, can_gc: CanGc) -> DomRoot<TrustedHTML> {
        TrustedHTML::new(DOMString::new(), &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-emptyscript>
    fn EmptyScript(&self, can_gc: CanGc) -> DomRoot<TrustedScript> {
        TrustedScript::new(DOMString::new(), &self.global(), can_gc)
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-getattributetype>
    fn GetAttributeType(
        &self,
        tag_name: DOMString,
        attribute: DOMString,
        element_namespace: Option<DOMString>,
        attribute_namespace: Option<DOMString>,
    ) -> Option<DOMString> {
        // Step 1: Set localName to tagName in ASCII lowercase.
        let local_name = tag_name.to_ascii_lowercase();
        // Step 2: Set attribute to attribute in ASCII lowercase.
        let attribute = attribute.to_ascii_lowercase();
        // Step 3: If elementNs is null or an empty string, set elementNs to HTML namespace.
        let element_namespace = match element_namespace {
            Some(namespace) if !namespace.is_empty() => Namespace::from(namespace),
            Some(_) | None => ns!(html),
        };
        // Step 4: If attrNs is an empty string, set attrNs to null.
        let attribute_namespace = match attribute_namespace {
            Some(namespace) if !namespace.is_empty() => Some(Namespace::from(namespace)),
            Some(_) | None => None,
        };
        // Step 5: Let interface be the element interface for localName and elementNs.
        // Step 6: Let expectedType be null.
        // Step 7: Set attributeData to the result of Get Trusted Type data for attribute algorithm,
        // with the following arguments: interface as element, attribute, attrNs
        // Step 8: If attributeData is not null, then set expectedType to the interface’s name of
        // the value of the fourth member of attributeData.
        // Step 9: Return expectedType.
        TrustedTypePolicyFactory::get_trusted_type_data_for_attribute(
            &element_namespace,
            &LocalName::from(local_name),
            &attribute,
            attribute_namespace.as_ref(),
        )
        .map(|tuple| DOMString::from(tuple.0.as_ref()))
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-getpropertytype>
    #[allow(clippy::if_same_then_else)]
    fn GetPropertyType(
        &self,
        tag_name: DOMString,
        property: DOMString,
        element_namespace: Option<DOMString>,
    ) -> Option<DOMString> {
        // Step 1: Set localName to tagName in ASCII lowercase.
        let local_name = tag_name.to_ascii_lowercase();
        // Step 2: If elementNs is null or an empty string, set elementNs to HTML namespace.
        let element_namespace = match element_namespace {
            Some(namespace) if !namespace.is_empty() => Namespace::from(namespace),
            Some(_) | None => ns!(html),
        };
        // Step 3: Let interface be the element interface for localName and elementNs.
        let interface = QualName::new(None, element_namespace, LocalName::from(local_name));
        // Step 4: Let expectedType be null.
        let mut expected_type = None;
        // Step 5: Find the row in the following table, where the first column is "*" or interface’s name,
        // and property is in the second column. If a matching row is found, set expectedType to
        // the interface’s name of the value of the third column.
        let property = property.str();
        if interface.ns == ns!(html) &&
            interface.local == local_name!("iframe") &&
            property == "srcdoc"
        {
            expected_type = Some(DOMString::from("TrustedHTML"))
        } else if interface.ns == ns!(html) &&
            interface.local == local_name!("script") &&
            property == "innerText"
        {
            expected_type = Some(DOMString::from("TrustedScript"))
        } else if interface.ns == ns!(html) &&
            interface.local == local_name!("script") &&
            property == "src"
        {
            expected_type = Some(DOMString::from("TrustedScriptURL"))
        } else if interface.ns == ns!(html) &&
            interface.local == local_name!("script") &&
            property == "text"
        {
            expected_type = Some(DOMString::from("TrustedScript"))
        } else if interface.ns == ns!(html) &&
            interface.local == local_name!("script") &&
            property == "textContent"
        {
            expected_type = Some(DOMString::from("TrustedScript"))
        } else if property == "innerHTML" {
            expected_type = Some(DOMString::from("TrustedHTML"))
        } else if property == "outerHTML" {
            expected_type = Some(DOMString::from("TrustedHTML"))
        }
        // Step 6: Return expectedType.
        expected_type
    }
    /// <https://www.w3.org/TR/trusted-types/#dom-trustedtypepolicyfactory-defaultpolicy>
    fn GetDefaultPolicy(&self) -> Option<DomRoot<TrustedTypePolicy>> {
        self.default_policy.get()
    }
}
