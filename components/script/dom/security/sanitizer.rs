/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use dom_struct::dom_struct;
use html5ever::{Namespace, ns};
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::SanitizerBinding::{
    SanitizerAttribute, SanitizerAttributeNamespace, SanitizerConfig, SanitizerElement,
    SanitizerElementNamespace, SanitizerElementNamespaceWithAttributes,
    SanitizerElementWithAttributes, SanitizerMethods, SanitizerPresets,
};
use crate::dom::bindings::codegen::UnionTypes::SanitizerConfigOrSanitizerPresets;
use crate::dom::bindings::domname::is_valid_attribute_local_name;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct Sanitizer {
    reflector_: Reflector,
    /// <https://wicg.github.io/sanitizer-api/#sanitizer-configuration>
    configuration: DomRefCell<SanitizerConfig>,
}

impl Sanitizer {
    fn new_inherited(configuration: SanitizerConfig) -> Sanitizer {
        Sanitizer {
            reflector_: Reflector::new(),
            configuration: DomRefCell::new(configuration),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        configuration: SanitizerConfig,
    ) -> DomRoot<Sanitizer> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(Sanitizer::new_inherited(configuration)),
            window,
            proto,
            cx,
        )
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-set-a-configuration>
    fn set_configuration(
        &self,
        mut configuration: SanitizerConfig,
        allow_comments_pis_and_data_attributes: bool,
    ) -> bool {
        // Step 1. Canonicalize configuration with allowCommentsPIsAndDataAttributes.
        configuration.canonicalize(allow_comments_pis_and_data_attributes);

        // Step 2. If configuration is not valid, then return false.
        if !configuration.is_valid() {
            return false;
        }

        // Step 3. Set sanitizer’s configuration to configuration.
        let mut sanitizer_configuration = self.configuration.borrow_mut();
        *sanitizer_configuration = configuration;

        // Step 4. Return true.
        true
    }
}

impl SanitizerMethods<crate::DomTypeHolder> for Sanitizer {
    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-constructor>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        configuration: SanitizerConfigOrSanitizerPresets,
    ) -> Fallible<DomRoot<Sanitizer>> {
        let configuration = match configuration {
            // Step 1. If configuration is a SanitizerPresets string, then:
            SanitizerConfigOrSanitizerPresets::SanitizerPresets(configuration) => {
                // Step 1.1. Assert: configuration is default.
                assert_eq!(configuration, SanitizerPresets::Default);

                // Step 1.2. Set configuration to the built-in safe default configuration.
                built_in_safe_default_configuration()
            },
            SanitizerConfigOrSanitizerPresets::SanitizerConfig(configuration) => configuration,
        };

        // Step 2. Let valid be the return value of set a configuration with configuration and true
        // on this.
        // Step 3. If valid is false, then throw a TypeError.
        let sanitizer = Sanitizer::new_with_proto(cx, window, proto, SanitizerConfig::default());
        if !sanitizer.set_configuration(configuration, true) {
            return Err(Error::Type(c"The configuration is invalid".into()));
        }

        Ok(sanitizer)
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-get>
    fn Get(&self) -> SanitizerConfig {
        // Step 1. Let config be this’s configuration.
        let config = self.configuration.borrow_mut();

        // TODO: Step 2 to Step 7

        // Step 8. Return config.
        (*config).clone()
    }
}

trait SanitizerConfigAlgorithm {
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-valid>
    fn is_valid(&self) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-canonicalize-the-configuration>
    fn canonicalize(&mut self, allow_comments_pis_and_data_attributes: bool);
}

impl SanitizerConfigAlgorithm for SanitizerConfig {
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-valid>
    fn is_valid(&self) -> bool {
        // NOTE: It’s expected that the configuration being passing in has previously been run
        // through the canonicalize the configuration steps. We will simply assert conditions that
        // that algorithm should have guaranteed to hold.

        // Step 1. Assert: config["elements"] exists or config["removeElements"] exists.
        assert!(self.elements.is_some() || self.removeElements.is_some());

        // Step 2. If config["elements"] exists and config["removeElements"] exists, then return
        // false.
        if self.elements.is_some() && self.removeElements.is_some() {
            return false;
        }

        // TODO:
        // Step 3. Assert: Either config["processingInstructions"] exists or
        // config["removeProcessingInstructions"] exists.
        // Step 4. If config["processingInstructions"] exists and
        // config["removeProcessingInstructions"] exists, then return false.

        // Step 5. Assert: Either config["attributes"] exists or config["removeAttributes"] exists.
        assert!(self.attributes.is_some() || self.removeAttributes.is_some());

        // Step 6. If config["attributes"] exists and config["removeAttributes"] exists, then return
        // false.
        if self.attributes.is_some() && self.removeAttributes.is_some() {
            return false;
        }

        // Step 7. Assert: All SanitizerElementNamespaceWithAttributes, SanitizerElementNamespace,
        // SanitizerProcessingInstruction, and SanitizerAttributeNamespace items in config are
        // canonical, meaning they have been run through canonicalize a sanitizer element,
        // canonicalize a sanitizer processing instruction, or canonicalize a sanitizer attribute,
        // as appropriate.
        //
        // NOTE: This assertion could be done by running the canonicalization again to see if there
        // is any changes. Since it is expected to canonicalize the configuration before running
        // this `is_valid` function, we simply skip this assert for the sake of performace.

        match &self.elements {
            // Step 8. If config["elements"] exists:
            Some(config_elements) => {
                // Step 8.1. If config["elements"] has duplicates, then return false.
                if config_elements.has_duplicates() {
                    return false;
                }
            },
            // Step 9. Otherwise:
            None => {
                // Step 9.1. If config["removeElements"] has duplicates, then return false.
                if self
                    .removeElements
                    .as_ref()
                    .is_some_and(|config_remove_elements| config_remove_elements.has_duplicates())
                {
                    return false;
                }
            },
        }

        // Step 10. If config["replaceWithChildrenElements"] exists and has duplicates, then return
        // false.
        if self
            .replaceWithChildrenElements
            .as_ref()
            .is_some_and(|replace_with_children_elements| {
                replace_with_children_elements.has_duplicates()
            })
        {
            return false;
        }

        // TODO:
        // Step 11. If config["processingInstructions"] exists:
        // Step 11.1. If config["processingInstructions"] has duplicate targets, then return false.
        // Step 12. Otherwise:
        // Step 12.1. If config["removeProcessingInstructions"] has duplicate targets, then return
        // false.

        match &self.attributes {
            // Step 13. If config["attributes"] exists:
            Some(config_attributes) => {
                // Step 13.1. If config["attributes"] has duplicates, then return false.
                if config_attributes.has_duplicates() {
                    return false;
                }
            },
            // Step 14. Otherwise:
            None => {
                // Step 14.1. If config["removeAttributes"] has duplicates, then return false.
                if self
                    .removeAttributes
                    .as_ref()
                    .is_some_and(|config_remove_attributes| {
                        config_remove_attributes.has_duplicates()
                    })
                {
                    return false;
                }
            },
        }

        // Step 15. If config["replaceWithChildrenElements"] exists:
        if let Some(config_replace_with_children_elements) = &self.replaceWithChildrenElements {
            // Step 15.1. For each element of config["replaceWithChildrenElements"]:
            for element in config_replace_with_children_elements {
                // Step 15.1.1. If the built-in non-replaceable elements list contains element, then
                // return false.
                if built_in_non_replaceable_elements_list().contains_name(element) {
                    return false;
                }
            }

            match &self.elements {
                // Step 15.2. If config["elements"] exists:
                Some(config_elements) => {
                    // Step 15.2.1. If the intersection of config["elements"] and
                    // config["replaceWithChildrenElements"] is not empty, then return false.
                    if config_elements
                        .is_intersection_non_empty(config_replace_with_children_elements)
                    {
                        return false;
                    }
                },
                // Step 15.3. Otherwise:
                None => {
                    // Step 15.3.1. If the intersection of config["removeElements"] and
                    // config["replaceWithChildrenElements"] is not empty, then return false.
                    if self
                        .removeElements
                        .as_ref()
                        .is_some_and(|config_remove_elements| {
                            config_remove_elements
                                .is_intersection_non_empty(config_replace_with_children_elements)
                        })
                    {
                        return false;
                    }
                },
            }
        }

        match &self.attributes {
            // Step 16. If config["attributes"] exists:
            Some(config_attributes) => {
                // Step 16.1. Assert: config["dataAttributes"] exists.
                assert!(self.dataAttributes.is_some());

                // Step 16.2. If config["elements"] exists:
                if let Some(config_elements) = &self.elements {
                    // Step 16.2.1. For each element of config["elements"]:
                    for element in config_elements {
                        // Step 16.2.1.1. If element["attributes"] exists and element["attributes"]
                        // has duplicates, then return false.
                        if element
                            .attributes()
                            .is_some_and(|element_attributes| element_attributes.has_duplicates())
                        {
                            return false;
                        }

                        // Step 16.2.1.2. If element["removeAttributes"] exists and
                        // element["removeAttributes"] has duplicates, then return false.
                        if element
                            .remove_attributes()
                            .is_some_and(|element_remove_attributes| {
                                element_remove_attributes.has_duplicates()
                            })
                        {
                            return false;
                        }

                        // Step 16.2.1.3. If the intersection of config["attributes"] and
                        // element["attributes"] with default « » is not empty, then return false.
                        if config_attributes
                            .is_intersection_non_empty(element.attributes().unwrap_or_default())
                        {
                            return false;
                        }

                        // Step 16.2.1.4. If element["removeAttributes"] with default « » is not a
                        // subset of config["attributes"], then return false.
                        if !element
                            .remove_attributes()
                            .unwrap_or_default()
                            .iter()
                            .all(|entry| config_attributes.contains_name(entry))
                        {
                            return false;
                        }

                        // Step 16.2.1.5. If config["dataAttributes"] is true and
                        // element["attributes"] contains a custom data attribute, then return
                        // false.
                        if self.dataAttributes == Some(true) &&
                            element.attributes().is_some_and(|attributes| {
                                attributes.iter().any(|attribute| {
                                    is_custom_data_attribute(
                                        &attribute.name().str(),
                                        attribute
                                            .namespace()
                                            .map(|namespace| namespace.str())
                                            .as_deref(),
                                    )
                                })
                            })
                        {
                            return false;
                        }
                    }
                }

                // Step 16.3. If config["dataAttributes"] is true and config["attributes"] contains
                // a custom data attribute, then return false.
                if self.dataAttributes == Some(true) &&
                    config_attributes.iter().any(|attribute| {
                        is_custom_data_attribute(
                            &attribute.name().str(),
                            attribute
                                .namespace()
                                .map(|namespace| namespace.str())
                                .as_deref(),
                        )
                    })
                {
                    return false;
                }
            },
            // Step 17. Otherwise:
            None => {
                // Step 17.1. If config["elements"] exists:
                if let Some(config_elements) = &self.elements {
                    // Step 17.1.1. For each element of config["elements"]:
                    for element in config_elements {
                        // Step 17.1.1.1. If element["attributes"] exists and
                        // element["removeAttributes"] exists, then return false.
                        if element.attributes().is_some() && element.remove_attributes().is_some() {
                            return false;
                        }

                        // Step 17.1.1.2. If element["attributes"] exist and element["attributes"]
                        // has duplicates, then return false.
                        if element
                            .attributes()
                            .as_ref()
                            .is_some_and(|element_attributes| element_attributes.has_duplicates())
                        {
                            return false;
                        }

                        // Step 17.1.1.3. If element["removeAttributes"] exist and
                        // element["removeAttributes"] has duplicates, then return false.
                        if element.remove_attributes().as_ref().is_some_and(
                            |element_remove_attributes| element_remove_attributes.has_duplicates(),
                        ) {
                            return false;
                        }

                        // Step 17.1.1.4. If the intersection of config["removeAttributes"] and
                        // element["attributes"] with default « » is not empty, then return false.
                        if self
                            .removeAttributes
                            .as_ref()
                            .is_some_and(|config_remove_attributes| {
                                config_remove_attributes.is_intersection_non_empty(
                                    element.attributes().unwrap_or_default(),
                                )
                            })
                        {
                            return false;
                        }

                        // Step 17.1.1.5. If the intersection of config["removeAttributes"] and
                        // element["removeAttributes"] with default « » is not empty, then return
                        // false.
                        if self
                            .removeAttributes
                            .as_ref()
                            .is_some_and(|config_remove_attributes| {
                                config_remove_attributes.is_intersection_non_empty(
                                    element.remove_attributes().unwrap_or_default(),
                                )
                            })
                        {
                            return false;
                        }
                    }
                }

                // Step 17.2. If config["dataAttributes"] exists, then return false.
                if self.dataAttributes.is_some() {
                    return false;
                }
            },
        }

        // Step 18. Return true.
        true
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-canonicalize-the-configuration>
    fn canonicalize(&mut self, allow_comments_pis_and_data_attributes: bool) {
        // Step 1. If neither configuration["elements"] nor configuration["removeElements"] exist,
        // then set configuration["removeElements"] to « ».
        if self.elements.is_none() && self.removeElements.is_none() {
            self.removeElements = Some(Vec::new());
        }

        // TODO:
        // Step 2. If neither configuration["processingInstructions"] nor
        // configuration["removeProcessingInstructions"] exist:
        // Step 2.1. If allowCommentsPIsAndDataAttributes is true, then set
        // configuration["removeProcessingInstructions"] to « ».
        // Step 2.2. Otherwise, set configuration["processingInstructions"] to « ».

        // Step 3. If neither configuration["attributes"] nor configuration["removeAttributes"]
        // exist, then set configuration["removeAttributes"] to « ».
        if self.attributes.is_none() && self.removeAttributes.is_none() {
            self.removeAttributes = Some(Vec::new());
        }

        // Step 4. If configuration["elements"] exists:
        if let Some(elements) = &mut self.elements {
            // Step 4.1. Let elements be « ».
            // Step 4.2. For each element of configuration["elements"] do:
            // Step 4.2.1. Append the result of canonicalize a sanitizer element with attributes
            // element to elements.
            // Step 4.3. Set configuration["elements"] to elements.
            *elements = elements
                .iter()
                .cloned()
                .map(SanitizerElementWithAttributes::canonicalize)
                .collect();
        }

        // Step 5. If configuration["removeElements"] exists:
        if let Some(remove_elements) = &mut self.removeElements {
            // Step 5.1. Let elements be « ».
            // Step 5.2. For each element of configuration["removeElements"] do:
            // Step 5.2.1. Append the result of canonicalize a sanitizer element element to
            // elements.
            // Step 5.3. Set configuration["removeElements"] to elements.
            *remove_elements = remove_elements
                .iter()
                .cloned()
                .map(SanitizerElement::canonicalize)
                .collect();
        }

        // Step 6. If configuration["replaceWithChildrenElements"] exists:
        if let Some(replace_with_children_elements) = &mut self.replaceWithChildrenElements {
            // Step 6.1. Let elements be « ».
            // Step 6.2. For each element of configuration["replaceWithChildrenElements"] do:
            // Step 6.2.1. Append the result of canonicalize a sanitizer element element to
            // elements.
            // Step 6.3. Set configuration["replaceWithChildrenElements"] to elements.
            *replace_with_children_elements = replace_with_children_elements
                .iter()
                .cloned()
                .map(SanitizerElement::canonicalize)
                .collect();
        }

        // TODO:
        // Step 7. If configuration["processingInstructions"] exists:
        // Step 7.1. Let processingInstructions be « ».
        // Step 7.2. For each pi of configuration["processingInstructions"]:
        // Step 7.2.1. Append the result of canonicalize a sanitizer processing instruction pi
        // to processingInstructions.
        // Step 7.3. Set configuration["processingInstructions"] to processingInstructions.

        // TODO:
        // Step 8. If configuration["removeProcessingInstructions"] exists:
        // Step 8.1. Let processingInstructions be « ».
        // Step 8.2. For each pi of configuration["removeProcessingInstructions"]:
        // Step 8.2.1. Append the result of canonicalize a sanitizer processing instruction
        // pi to processingInstructions.
        // Step 8.3. Set configuration["removeProcessingInstructions"] to processingInstructions.

        // Step 9. If configuration["attributes"] exists:
        if let Some(attributes) = &mut self.attributes {
            // Step 9.1. Let attributes be « ».
            // Step 9.2. For each attribute of configuration["attributes"] do:
            // Step 9.2.1. Append the result of canonicalize a sanitizer attribute attribute to
            // attributes.
            // Step 9.3. Set configuration["attributes"] to attributes.
            *attributes = attributes
                .iter()
                .cloned()
                .map(SanitizerAttribute::canonicalize)
                .collect();
        }

        // Step 10. If configuration["removeAttributes"] exists:
        if let Some(remove_attributes) = &mut self.removeAttributes {
            // Step 10.1. Let attributes be « ».
            // Step 10.2. For each attribute of configuration["removeAttributes"] do:
            // Step 10.2.1. Append the result of canonicalize a sanitizer attribute attribute to
            // attributes.
            // Step 10.3. Set configuration["removeAttributes"] to attributes.
            *remove_attributes = remove_attributes
                .iter()
                .cloned()
                .map(SanitizerAttribute::canonicalize)
                .collect();
        }

        // Step 11. If configuration["comments"] does not exist, then set configuration["comments"]
        // to allowCommentsPIsAndDataAttributes.
        if self.comments.is_none() {
            self.comments = Some(allow_comments_pis_and_data_attributes);
        }

        // Step 12. If configuration["attributes"] exists and configuration["dataAttributes"] does
        // not exist, then set configuration["dataAttributes"] to allowCommentsPIsAndDataAttributes.
        if self.attributes.is_some() && self.dataAttributes.is_none() {
            self.dataAttributes = Some(allow_comments_pis_and_data_attributes);
        }
    }
}

trait Canonicalization {
    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-element-with-attributes>
    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-element>
    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-attribute>
    fn canonicalize(self) -> Self;
}

impl Canonicalization for SanitizerElementWithAttributes {
    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-element-with-attributes>
    fn canonicalize(mut self) -> Self {
        // Step 1. Let result be the result of canonicalize a sanitizer element with element.
        let parent = match &mut self {
            SanitizerElementWithAttributes::String(name) => {
                SanitizerElement::String(std::mem::take(name))
            },
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace {
                    name: std::mem::take(&mut dictionary.parent.name),
                    namespace: dictionary.parent.namespace.as_mut().map(std::mem::take),
                })
            },
        };
        let mut canonicalized_parent = parent.canonicalize();
        let mut result = SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(
            SanitizerElementNamespaceWithAttributes {
                parent: SanitizerElementNamespace {
                    name: std::mem::take(canonicalized_parent.name_mut()),
                    namespace: canonicalized_parent.namespace_mut().map(std::mem::take),
                },
                attributes: None,
                removeAttributes: None,
            },
        );

        // Step 2. If element is a dictionary:
        if matches!(
            self,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(_)
        ) {
            // Step 2.1. If element["attributes"] exists:
            if let Some(attributes) = self.attributes() {
                // Step 2.1.1. Let attributes be « ».
                // Step 2.1.2. For each attribute of element["attributes"]:
                // Step 2.1.2.1. Append the result of canonicalize a sanitizer attribute with
                // attribute to attributes.
                let attributes = attributes
                    .iter()
                    .cloned()
                    .map(|attribute| attribute.canonicalize())
                    .collect();

                // Step 2.1.3. Set result["attributes"] to attributes.
                result.set_attributes(Some(attributes));
            }

            // Step 2.2. If element["removeAttributes"] exists:
            if let Some(remove_attributes) = self.remove_attributes() {
                // Step 2.2.1. Let attributes be « ».
                // Step 2.2.2. For each attribute of element["removeAttributes"]:
                // Step 2.2.2.1. Append the result of canonicalize a sanitizer attribute with
                // attribute to attributes.
                let attributes = remove_attributes
                    .iter()
                    .cloned()
                    .map(|attribute| attribute.canonicalize())
                    .collect();

                // Step 2.2.3. Set result["removeAttributes"] to attributes.
                result.set_remove_attributes(Some(attributes));
            }
        }

        // Step 3. If neither result["attributes"] nor result["removeAttributes"] exist:
        if result.attributes().is_none() && result.remove_attributes().is_none() {
            // Step 3.1. Set result["removeAttributes"] to « ».
            result.set_remove_attributes(Some(Vec::new()));
        }

        // Step 4. Return result.
        result
    }
}

impl Canonicalization for SanitizerElement {
    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-element>
    fn canonicalize(self) -> Self {
        // Return the result of canonicalize a sanitizer name with element and the HTML namespace as
        // the default namespace.
        self.canonicalize_name(Some(ns!(html).to_string()))
    }
}

impl Canonicalization for SanitizerAttribute {
    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-attribute>
    fn canonicalize(self) -> Self {
        // Return the result of canonicalize a sanitizer name with attribute and null as the default
        // namespace.
        self.canonicalize_name(None)
    }
}

trait NameCanonicalization: NameMember {
    fn new_dictionary(name: DOMString, namespace: Option<DOMString>) -> Self;
    fn is_string(&self) -> bool;
    fn is_dictionary(&self) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#canonicalize-a-sanitizer-name>
    fn canonicalize_name(mut self, default_namespace: Option<String>) -> Self {
        // Step 1. Assert: name is either a DOMString or a dictionary.
        assert!(self.is_string() || self.is_dictionary());

        // Step 2. If name is a DOMString, then return «[ "name" → name, "namespace" →
        // defaultNamespace]».
        if self.is_string() {
            return Self::new_dictionary(
                std::mem::take(self.name_mut()),
                default_namespace.map(DOMString::from),
            );
        }

        // Step 3. Assert: name is a dictionary and both name["name"] and name["namespace"] exist.
        // NOTE: The latter is guaranteed by Rust type system.
        assert!(self.is_dictionary());

        // Step 4. If name["namespace"] is the empty string, then set it to null.
        if self
            .namespace()
            .is_some_and(|namespace| namespace.str() == "")
        {
            self.set_namespace(None);
        }

        // Step 5. Return «[
        // "name" → name["name"],
        // "namespace" → name["namespace"]
        // ]».
        Self::new_dictionary(
            std::mem::take(self.name_mut()),
            self.namespace_mut().map(std::mem::take),
        )
    }
}

impl NameCanonicalization for SanitizerElement {
    fn new_dictionary(name: DOMString, namespace: Option<DOMString>) -> Self {
        SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace { name, namespace })
    }

    fn is_string(&self) -> bool {
        matches!(self, SanitizerElement::String(_))
    }

    fn is_dictionary(&self) -> bool {
        matches!(self, SanitizerElement::SanitizerElementNamespace(_))
    }
}

impl NameCanonicalization for SanitizerAttribute {
    fn new_dictionary(name: DOMString, namespace: Option<DOMString>) -> Self {
        SanitizerAttribute::SanitizerAttributeNamespace(SanitizerAttributeNamespace {
            name,
            namespace,
        })
    }

    fn is_string(&self) -> bool {
        matches!(self, SanitizerAttribute::String(_))
    }

    fn is_dictionary(&self) -> bool {
        matches!(self, SanitizerAttribute::SanitizerAttributeNamespace(_))
    }
}

/// Supporting algorithms on lists of elements and lists of attributes, from the specification.
trait NameSlice<T>
where
    T: NameMember + Canonicalization + Clone,
{
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-contains>
    fn contains_name<S: NameMember>(&self, other: &S) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-has-duplicates>
    fn has_duplicates(&self) -> bool;

    /// Custom version of the supporting algorithm
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-intersection> that checks whether the
    /// intersection is non-empty, returning early if it is non-empty for efficiency.
    fn is_intersection_non_empty<S>(&self, others: &[S]) -> bool
    where
        S: NameMember + Canonicalization + Clone;
}

impl<T> NameSlice<T> for [T]
where
    T: NameMember + Canonicalization + Clone,
{
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-contains>
    fn contains_name<S: NameMember>(&self, other: &S) -> bool {
        // A Sanitizer name list contains an item if there exists an entry of list that is an
        // ordered map, and where item["name"] equals entry["name"] and item["namespace"] equals
        // entry["namespace"].
        self.iter()
            .any(|entry| entry.name() == other.name() && entry.namespace() == other.namespace())
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-has-duplicates>
    fn has_duplicates(&self) -> bool {
        // A list list has duplicates, if for any item of list, there is more than one entry in list
        // where item["name"] is entry["name"] and item["namespace"] is entry["namespace"].
        let mut used = HashSet::new();
        self.iter().any(move |entry| {
            !used.insert((
                entry.name().to_string(),
                entry.namespace().map(DOMString::to_string),
            ))
        })
    }

    /// Custom version of the supporting algorithm
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-intersection> that checks whether the
    /// intersection is non-empty, returning early if it is non-empty for efficiency.
    fn is_intersection_non_empty<S>(&self, others: &[S]) -> bool
    where
        S: NameMember + Canonicalization + Clone,
    {
        // Step 1. Let set A be « [] ».
        // Step 2. Let set B be « [] ».
        // Step 3. For each entry of A, append the result of canonicalize a sanitizer name entry to
        // set A.
        // Step 4. For each entry of B, append the result of canonicalize a sanitizer name entry to
        // set B.
        let a = self.iter().map(|entry| entry.clone().canonicalize());
        let b = others
            .iter()
            .map(|entry| entry.clone().canonicalize())
            .collect::<Vec<S>>();

        // Step 5. Return the intersection of set A and set B.
        // NOTE: Instead of returning the intersection itself, return true if the intersection is
        // non-empty, and false otherwise.
        a.filter(|entry| {
            b.iter()
                .any(|other| entry.name() == other.name() && entry.namespace() == other.namespace())
        })
        .any(|_| true)
    }
}

/// Helper functions for accessing the "name" and "namespace" members of
/// [`SanitizerElementWithAttributes`], [`SanitizerElement`] and [`SanitizerAttribute`].
trait NameMember: Sized {
    fn name(&self) -> &DOMString;
    fn name_mut(&mut self) -> &mut DOMString;
    fn namespace(&self) -> Option<&DOMString>;
    fn namespace_mut(&mut self) -> Option<&mut DOMString>;

    fn set_namespace(&mut self, namespace: Option<&str>);
}

impl NameMember for SanitizerElementWithAttributes {
    fn name(&self) -> &DOMString {
        match self {
            SanitizerElementWithAttributes::String(name) => name,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                &dictionary.parent.name
            },
        }
    }

    fn name_mut(&mut self) -> &mut DOMString {
        match self {
            SanitizerElementWithAttributes::String(name) => name,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                &mut dictionary.parent.name
            },
        }
    }

    fn namespace(&self) -> Option<&DOMString> {
        match self {
            SanitizerElementWithAttributes::String(_) => None,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.parent.namespace.as_ref()
            },
        }
    }

    fn namespace_mut(&mut self) -> Option<&mut DOMString> {
        match self {
            SanitizerElementWithAttributes::String(_) => None,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.parent.namespace.as_mut()
            },
        }
    }

    fn set_namespace(&mut self, namespace: Option<&str>) {
        match self {
            SanitizerElementWithAttributes::String(name) => {
                let new_instance =
                    SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(
                        SanitizerElementNamespaceWithAttributes {
                            parent: SanitizerElementNamespace {
                                name: std::mem::take(name),
                                namespace: namespace.map(DOMString::from),
                            },
                            attributes: None,
                            removeAttributes: None,
                        },
                    );
                *self = new_instance;
            },
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.parent.namespace = namespace.map(DOMString::from);
            },
        }
    }
}

impl NameMember for SanitizerElement {
    fn name(&self) -> &DOMString {
        match self {
            SanitizerElement::String(name) => name,
            SanitizerElement::SanitizerElementNamespace(dictionary) => &dictionary.name,
        }
    }

    fn name_mut(&mut self) -> &mut DOMString {
        match self {
            SanitizerElement::String(name) => name,
            SanitizerElement::SanitizerElementNamespace(dictionary) => &mut dictionary.name,
        }
    }

    fn namespace(&self) -> Option<&DOMString> {
        match self {
            SanitizerElement::String(_) => None,
            SanitizerElement::SanitizerElementNamespace(dictionary) => {
                dictionary.namespace.as_ref()
            },
        }
    }

    fn namespace_mut(&mut self) -> Option<&mut DOMString> {
        match self {
            SanitizerElement::String(_) => None,
            SanitizerElement::SanitizerElementNamespace(dictionary) => {
                dictionary.namespace.as_mut()
            },
        }
    }

    fn set_namespace(&mut self, namespace: Option<&str>) {
        match self {
            SanitizerElement::String(name) => {
                let new_instance =
                    SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace {
                        name: std::mem::take(name),
                        namespace: namespace.map(DOMString::from),
                    });
                *self = new_instance;
            },
            SanitizerElement::SanitizerElementNamespace(dictionary) => {
                dictionary.namespace = namespace.map(DOMString::from);
            },
        }
    }
}

impl NameMember for SanitizerAttribute {
    fn name(&self) -> &DOMString {
        match self {
            SanitizerAttribute::String(name) => name,
            SanitizerAttribute::SanitizerAttributeNamespace(dictionary) => &dictionary.name,
        }
    }

    fn name_mut(&mut self) -> &mut DOMString {
        match self {
            SanitizerAttribute::String(name) => name,
            SanitizerAttribute::SanitizerAttributeNamespace(dictionary) => &mut dictionary.name,
        }
    }

    fn namespace(&self) -> Option<&DOMString> {
        match self {
            SanitizerAttribute::String(_) => None,
            SanitizerAttribute::SanitizerAttributeNamespace(dictionary) => {
                dictionary.namespace.as_ref()
            },
        }
    }

    fn namespace_mut(&mut self) -> Option<&mut DOMString> {
        match self {
            SanitizerAttribute::String(_) => None,
            SanitizerAttribute::SanitizerAttributeNamespace(dictionary) => {
                dictionary.namespace.as_mut()
            },
        }
    }

    fn set_namespace(&mut self, namespace: Option<&str>) {
        match self {
            SanitizerAttribute::String(name) => {
                let new_instance =
                    SanitizerAttribute::SanitizerAttributeNamespace(SanitizerAttributeNamespace {
                        name: std::mem::take(name),
                        namespace: namespace.map(DOMString::from),
                    });
                *self = new_instance;
            },
            SanitizerAttribute::SanitizerAttributeNamespace(dictionary) => {
                dictionary.namespace = namespace.map(DOMString::from);
            },
        }
    }
}

/// Helper functions for accessing the "attributes" and "removeAttributes" members of
/// [`SanitizerElementWithAttributes`].
trait AttributeMember {
    fn attributes(&self) -> Option<&[SanitizerAttribute]>;
    fn remove_attributes(&self) -> Option<&[SanitizerAttribute]>;

    fn set_attributes(&mut self, attributes: Option<Vec<SanitizerAttribute>>);
    fn set_remove_attributes(&mut self, remove_attributes: Option<Vec<SanitizerAttribute>>);
}

impl AttributeMember for SanitizerElementWithAttributes {
    fn attributes(&self) -> Option<&[SanitizerAttribute]> {
        match self {
            SanitizerElementWithAttributes::String(_) => None,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.attributes.as_deref()
            },
        }
    }

    fn remove_attributes(&self) -> Option<&[SanitizerAttribute]> {
        match self {
            SanitizerElementWithAttributes::String(_) => None,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.removeAttributes.as_deref()
            },
        }
    }

    fn set_attributes(&mut self, attributes: Option<Vec<SanitizerAttribute>>) {
        match self {
            SanitizerElementWithAttributes::String(name) => {
                *self = SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(
                    SanitizerElementNamespaceWithAttributes {
                        parent: SanitizerElementNamespace {
                            name: std::mem::take(name),
                            namespace: None,
                        },
                        attributes,
                        removeAttributes: None,
                    },
                );
            },
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.attributes = attributes;
            },
        }
    }

    fn set_remove_attributes(&mut self, remove_attributes: Option<Vec<SanitizerAttribute>>) {
        match self {
            SanitizerElementWithAttributes::String(name) => {
                *self = SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(
                    SanitizerElementNamespaceWithAttributes {
                        parent: SanitizerElementNamespace {
                            name: std::mem::take(name),
                            namespace: None,
                        },
                        attributes: None,
                        removeAttributes: remove_attributes,
                    },
                );
            },
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.removeAttributes = remove_attributes;
            },
        }
    }
}

/// <https://wicg.github.io/sanitizer-api/#built-in-safe-default-configuration>
fn built_in_safe_default_configuration() -> SanitizerConfig {
    const ELEMENTS: &[(&str, &Namespace, &[&str])] = &[
        ("math", &ns!(mathml), &[]),
        ("merror", &ns!(mathml), &[]),
        ("mfrac", &ns!(mathml), &[]),
        ("mi", &ns!(mathml), &[]),
        ("mmultiscripts", &ns!(mathml), &[]),
        ("mn", &ns!(mathml), &[]),
        (
            "mo",
            &ns!(mathml),
            &[
                "fence",
                "form",
                "largeop",
                "lspace",
                "maxsize",
                "minsize",
                "movablelimits",
                "rspace",
                "separator",
                "stretchy",
                "symmetric",
            ],
        ),
        ("mover", &ns!(mathml), &["accent"]),
        (
            "mpadded",
            &ns!(mathml),
            &["depth", "height", "lspace", "voffset", "width"],
        ),
        ("mphantom", &ns!(mathml), &[]),
        ("mprescripts", &ns!(mathml), &[]),
        ("mroot", &ns!(mathml), &[]),
        ("mrow", &ns!(mathml), &[]),
        ("ms", &ns!(mathml), &[]),
        ("mspace", &ns!(mathml), &["depth", "height", "width"]),
        ("msqrt", &ns!(mathml), &[]),
        ("mstyle", &ns!(mathml), &[]),
        ("msub", &ns!(mathml), &[]),
        ("msubsup", &ns!(mathml), &[]),
        ("msup", &ns!(mathml), &[]),
        ("mtable", &ns!(mathml), &[]),
        ("mtd", &ns!(mathml), &["columnspan", "rowspan"]),
        ("mtext", &ns!(mathml), &[]),
        ("mtr", &ns!(mathml), &[]),
        ("munder", &ns!(mathml), &["accentunder"]),
        ("munderover", &ns!(mathml), &["accent", "accentunder"]),
        ("semantics", &ns!(mathml), &[]),
        ("a", &ns!(html), &["href", "hreflang", "type"]),
        ("abbr", &ns!(html), &[]),
        ("address", &ns!(html), &[]),
        ("article", &ns!(html), &[]),
        ("aside", &ns!(html), &[]),
        ("b", &ns!(html), &[]),
        ("bdi", &ns!(html), &[]),
        ("bdo", &ns!(html), &[]),
        ("blockquote", &ns!(html), &["cite"]),
        ("body", &ns!(html), &[]),
        ("br", &ns!(html), &[]),
        ("caption", &ns!(html), &[]),
        ("cite", &ns!(html), &[]),
        ("code", &ns!(html), &[]),
        ("col", &ns!(html), &["span"]),
        ("colgroup", &ns!(html), &["span"]),
        ("data", &ns!(html), &["value"]),
        ("dd", &ns!(html), &[]),
        ("del", &ns!(html), &["cite", "datetime"]),
        ("dfn", &ns!(html), &[]),
        ("div", &ns!(html), &[]),
        ("dl", &ns!(html), &[]),
        ("dt", &ns!(html), &[]),
        ("em", &ns!(html), &[]),
        ("figcaption", &ns!(html), &[]),
        ("figure", &ns!(html), &[]),
        ("footer", &ns!(html), &[]),
        ("h1", &ns!(html), &[]),
        ("h2", &ns!(html), &[]),
        ("h3", &ns!(html), &[]),
        ("h4", &ns!(html), &[]),
        ("h5", &ns!(html), &[]),
        ("h6", &ns!(html), &[]),
        ("head", &ns!(html), &[]),
        ("header", &ns!(html), &[]),
        ("hgroup", &ns!(html), &[]),
        ("hr", &ns!(html), &[]),
        ("html", &ns!(html), &[]),
        ("i", &ns!(html), &[]),
        ("ins", &ns!(html), &["cite", "datetime"]),
        ("kbd", &ns!(html), &[]),
        ("li", &ns!(html), &["value"]),
        ("main", &ns!(html), &[]),
        ("mark", &ns!(html), &[]),
        ("menu", &ns!(html), &[]),
        ("nav", &ns!(html), &[]),
        ("ol", &ns!(html), &["reversed", "start", "type"]),
        ("p", &ns!(html), &[]),
        ("pre", &ns!(html), &[]),
        ("q", &ns!(html), &[]),
        ("rp", &ns!(html), &[]),
        ("rt", &ns!(html), &[]),
        ("ruby", &ns!(html), &[]),
        ("s", &ns!(html), &[]),
        ("samp", &ns!(html), &[]),
        ("search", &ns!(html), &[]),
        ("section", &ns!(html), &[]),
        ("small", &ns!(html), &[]),
        ("span", &ns!(html), &[]),
        ("strong", &ns!(html), &[]),
        ("sub", &ns!(html), &[]),
        ("sup", &ns!(html), &[]),
        ("table", &ns!(html), &[]),
        ("tbody", &ns!(html), &[]),
        ("td", &ns!(html), &["colspan", "headers", "rowspan"]),
        ("tfoot", &ns!(html), &[]),
        (
            "th",
            &ns!(html),
            &["abbr", "colspan", "headers", "rowspan", "scope"],
        ),
        ("thead", &ns!(html), &[]),
        ("time", &ns!(html), &["datetime"]),
        ("title", &ns!(html), &[]),
        ("tr", &ns!(html), &[]),
        ("u", &ns!(html), &[]),
        ("ul", &ns!(html), &[]),
        ("var", &ns!(html), &[]),
        ("wbr", &ns!(html), &[]),
        ("a", &ns!(svg), &["href", "hreflang", "type"]),
        ("circle", &ns!(svg), &["cx", "cy", "pathLength", "r"]),
        ("defs", &ns!(svg), &[]),
        ("desc", &ns!(svg), &[]),
        (
            "ellipse",
            &ns!(svg),
            &["cx", "cy", "pathLength", "rx", "ry"],
        ),
        ("foreignObject", &ns!(svg), &["height", "width", "x", "y"]),
        ("g", &ns!(svg), &[]),
        ("line", &ns!(svg), &["pathLength", "x1", "x2", "y1", "y2"]),
        (
            "marker",
            &ns!(svg),
            &[
                "markerHeight",
                "markerUnits",
                "markerWidth",
                "orient",
                "preserveAspectRatio",
                "refX",
                "refY",
                "viewBox",
            ],
        ),
        ("metadata", &ns!(svg), &[]),
        ("path", &ns!(svg), &["d", "pathLength"]),
        ("polygon", &ns!(svg), &["pathLength", "points"]),
        ("polyline", &ns!(svg), &["pathLength", "points"]),
        (
            "rect",
            &ns!(svg),
            &["height", "pathLength", "rx", "ry", "width", "x", "y"],
        ),
        (
            "svg",
            &ns!(svg),
            &[
                "height",
                "preserveAspectRatio",
                "viewBox",
                "width",
                "x",
                "y",
            ],
        ),
        (
            "text",
            &ns!(svg),
            &["dx", "dy", "lengthAdjust", "rotate", "textLength", "x", "y"],
        ),
        (
            "textPath",
            &ns!(svg),
            &[
                "lengthAdjust",
                "method",
                "path",
                "side",
                "spacing",
                "startOffset",
                "textLength",
            ],
        ),
        ("title", &ns!(svg), &[]),
        (
            "tspan",
            &ns!(svg),
            &["dx", "dy", "lengthAdjust", "rotate", "textLength", "x", "y"],
        ),
    ];
    const ATTRIBUTES: &[&str] = &[
        "alignment-baseline",
        "baseline-shift",
        "clip-path",
        "clip-rule",
        "color",
        "color-interpolation",
        "cursor",
        "dir",
        "direction",
        "display",
        "displaystyle",
        "dominant-baseline",
        "fill",
        "fill-opacity",
        "fill-rule",
        "font-family",
        "font-size",
        "font-size-adjust",
        "font-stretch",
        "font-style",
        "font-variant",
        "font-weight",
        "lang",
        "letter-spacing",
        "marker-end",
        "marker-mid",
        "marker-start",
        "mathbackground",
        "mathcolor",
        "mathsize",
        "opacity",
        "paint-order",
        "pointer-events",
        "scriptlevel",
        "shape-rendering",
        "stop-color",
        "stop-opacity",
        "stroke",
        "stroke-dasharray",
        "stroke-dashoffset",
        "stroke-linecap",
        "stroke-linejoin",
        "stroke-miterlimit",
        "stroke-opacity",
        "stroke-width",
        "text-anchor",
        "text-decoration",
        "text-overflow",
        "text-rendering",
        "title",
        "transform",
        "transform-origin",
        "unicode-bidi",
        "vector-effect",
        "visibility",
        "white-space",
        "word-spacing",
        "writing-mode",
    ];

    let create_attribute_vec = |attributes: &[&str]| -> Vec<SanitizerAttribute> {
        attributes
            .iter()
            .map(|&attribute| {
                SanitizerAttribute::SanitizerAttributeNamespace(SanitizerAttributeNamespace {
                    name: attribute.into(),
                    namespace: None,
                })
            })
            .collect()
    };

    let elements = ELEMENTS
        .iter()
        .map(|&(name, namespace, attributes)| {
            let attributes = create_attribute_vec(attributes);
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(
                SanitizerElementNamespaceWithAttributes {
                    parent: SanitizerElementNamespace {
                        name: name.into(),
                        namespace: Some(namespace.to_string().into()),
                    },
                    attributes: Some(attributes),
                    removeAttributes: None,
                },
            )
        })
        .collect();

    let attributes = create_attribute_vec(ATTRIBUTES);

    SanitizerConfig {
        elements: Some(elements),
        removeElements: None,
        replaceWithChildrenElements: None,
        attributes: Some(attributes),
        removeAttributes: None,
        comments: Some(false),
        dataAttributes: Some(false),
    }
}

/// <https://wicg.github.io/sanitizer-api/#built-in-non-replaceable-elements-list>
fn built_in_non_replaceable_elements_list() -> Vec<SanitizerElement> {
    vec![
        SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace {
            name: "html".into(),
            namespace: Some(ns!(html).to_string().into()),
        }),
        SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace {
            name: "svg".into(),
            namespace: Some(ns!(svg).to_string().into()),
        }),
        SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace {
            name: "mathml".into(),
            namespace: Some(ns!(mathml).to_string().into()),
        }),
    ]
}

/// <https://html.spec.whatwg.org/multipage/#custom-data-attribute>
fn is_custom_data_attribute(name: &str, namespace: Option<&str>) -> bool {
    // A custom data attribute is an attribute in no namespace whose name starts with the string
    // "data-", has at least one character after the hyphen, is a valid attribute local name,
    // and contains no ASCII upper alphas.
    namespace.is_none() &&
        name.strip_prefix("data-")
            .is_some_and(|substring| !substring.is_empty()) &&
        is_valid_attribute_local_name(name) &&
        name.chars()
            .all(|code_point| !code_point.is_ascii_uppercase())
}
