/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::collections::HashSet;

use dom_struct::dom_struct;
use html5ever::{Namespace, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};

use crate::dom::bindings::codegen::Bindings::SanitizerBinding::{
    SanitizerAttribute, SanitizerAttributeNamespace, SanitizerConfig, SanitizerElement,
    SanitizerElementNamespace, SanitizerElementNamespaceWithAttributes,
    SanitizerElementWithAttributes, SanitizerMethods, SanitizerPresets,
};
use crate::dom::bindings::codegen::UnionTypes::SanitizerConfigOrSanitizerPresets;
use crate::dom::bindings::domname::is_valid_attribute_local_name;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::CONTENT_EVENT_HANDLER_NAMES;
use crate::dom::types::Console;
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
        let mut config = self.configuration.borrow_mut();

        // Step 2. Assert: config is valid.
        debug_assert!(config.is_valid());

        match &mut config.elements {
            // Step 3. If config["elements"] exists:
            Some(config_elements) => {
                // Step 3.1. For any element of config["elements"]:
                for element in config_elements.iter_mut() {
                    // Step 3.1.1. If element["attributes"] exists:
                    if let Some(element_attributes) = &mut element.attributes_mut() {
                        // Step 3.1.1.1. Set element["attributes"] to the result of sort in
                        // ascending order element["attributes"], with attrA being less than item
                        // attrB.
                        element_attributes.sort_by(|item_a, item_b| item_a.compare(item_b));
                    }

                    // Step 3.1.2. If element["removeAttributes"] exists:
                    if let Some(element_remove_attributes) = &mut element.remove_attributes_mut() {
                        // Step 3.1.2.1. Set element["removeAttributes"] to the result of sort in
                        // ascending order element["removeAttributes"], with attrA being less than
                        // item attrB.
                        element_remove_attributes.sort_by(|item_a, item_b| item_a.compare(item_b));
                    }
                }

                // Step 3.2. Set config["elements"] to the result of sort in ascending order
                // config["elements"], with elementA being less than item elementB.
                config_elements.sort_by(|item_a, item_b| item_a.compare(item_b));
            },
            // Step 4. Otherwise:
            None => {
                // Step 4.1. Set config["removeElements"] to the result of sort in ascending order
                // config["removeElements"], with elementA being less than item elementB.
                if let Some(config_remove_elements) = &mut config.removeElements {
                    config_remove_elements.sort_by(|item_a, item_b| item_a.compare(item_b));
                }
            },
        }

        // Step 5. If config["replaceWithChildrenElements"] exists:
        if let Some(config_replace_with_children_elements) = &mut config.replaceWithChildrenElements
        {
            // Step 5.1.Set config["replaceWithChildrenElements"] to the result of sort in ascending
            // order config["replaceWithChildrenElements"], with elementA being less than item
            // elementB.
            config_replace_with_children_elements.sort_by(|item_a, item_b| item_a.compare(item_b));
        }

        // TODO:
        // Step 6. If config["processingInstructions"] exists:
        // Step 6.1. Set config["processingInstructions"] to the result of sort in ascending order
        // config["processingInstructions"], with piA["target"] being code unit less than
        // piB["target"].
        // Step 7. Otherwise:
        // Step 7.1. Set config["removeProcessingInstructions"] to the result of sort in ascending
        // order config["removeProcessingInstructions"], with piA["target"] being code unit less
        // than piB["target"].

        match &mut config.attributes {
            // Step 8. If config["attributes"] exists:
            Some(config_attributes) => {
                // Step 8.1. Set config["attributes"] to the result of sort in ascending order
                // config["attributes"], with attrA being less than item attrB.
                config_attributes.sort_by(|item_a, item_b| item_a.compare(item_b));
            },
            // Step 9. Otherwise:
            None => {
                // Step 9.1. Set config["removeAttributes"] to the result of sort in ascending order
                // config["removeAttributes"], with attrA being less than item attrB.
                if let Some(config_remove_attributes) = &mut config.removeAttributes {
                    config_remove_attributes.sort_by(|item_a, item_b| item_a.compare(item_b));
                }
            },
        }

        // Step 10. Return config.
        (*config).clone()
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-allowelement>
    fn AllowElement(&self, element: SanitizerElementWithAttributes) -> bool {
        // Step 1. Let configuration be this’s configuration.
        let mut configuration = self.configuration.borrow_mut();

        // Step 2. Assert: configuration is valid.
        debug_assert!(configuration.is_valid());

        // Step 3. Set element to the result of canonicalize a sanitizer element with attributes
        // with element.
        let mut element = element.canonicalize();

        // Step 4. If configuration["elements"] exists:
        if configuration.elements.is_some() {
            // Step 4.1. Set modified to the result of remove element from
            // configuration["replaceWithChildrenElements"].
            let modified = if let Some(replace_with_children_elements) =
                &mut configuration.replaceWithChildrenElements
            {
                replace_with_children_elements.remove_item(&element)
            } else {
                false
            };

            // Step 4.2. Comment: We need to make sure the per-element attributes do not overlap
            // with global attributes.

            match &configuration.attributes {
                // Step 4.3. If configuration["attributes"] exists:
                Some(configuration_attributes) => {
                    // Step 4.3.1. If element["attributes"] exists:
                    if let Some(element_attributes) = element.attributes_mut() {
                        // Step 4.3.1.1. Set element["attributes"] to remove duplicates from
                        // element["attributes"].
                        element_attributes.remove_duplicates();

                        // Step 4.3.1.2. Set element["attributes"] to the difference of
                        // element["attributes"] and configuration["attributes"].
                        element_attributes.difference(configuration_attributes);

                        // Step 4.3.1.3. If configuration["dataAttributes"] is true:
                        if configuration.dataAttributes == Some(true) {
                            // Step 4.3.1.3.1. Remove all items item from element["attributes"]
                            // where item is a custom data attribute.
                            element_attributes.retain(|attribute| {
                                !is_custom_data_attribute(
                                    &attribute.name().str(),
                                    attribute
                                        .namespace()
                                        .map(|namespace| namespace.str())
                                        .as_deref(),
                                )
                            });
                        }
                    }

                    // Step 4.3.2. If element["removeAttributes"] exists:
                    if let Some(element_remove_attributes) = element.remove_attributes_mut() {
                        // Step 4.3.2.1. set element["removeattributes"] to remove duplicates from
                        // element["removeattributes"].
                        element_remove_attributes.remove_duplicates();

                        // Step 4.3.2.2. set element["removeattributes"] to the intersection of
                        // element["removeattributes"] and configuration["attributes"].
                        element_remove_attributes.intersection(configuration_attributes);
                    }
                },
                // Step 4.4. Otherwise:
                None => {
                    // NOTE: To avoid borrowing `element` again at Step 4.4.1.2 and 4.4.1.3 after
                    // borrowing `element` mutably at the beginning of Step 4.4.1, we clone
                    // element["attributes"] first, and call `set_attributes` at the end of Step
                    // 4.4.1 to put it back into `element`.

                    // Step 4.4.1. If element["attributes"] exists:
                    if let Some(mut element_attributes) = element.attributes_mut().cloned() {
                        // Step 4.4.1.1. Set element["attributes"] to remove duplicates from
                        // element["attributes"].
                        element_attributes.remove_duplicates();

                        // Step 4.4.1.2. Set element["attributes"] to the difference of
                        // element["attributes"] and element["removeAttributes"] with default « ».
                        element_attributes
                            .difference(element.remove_attributes().unwrap_or_default());

                        // Step 4.4.1.3. Remove element["removeAttributes"].
                        element.set_remove_attributes(None);

                        // Step 4.4.1.4. Set element["attributes"] to the difference of
                        // element["attributes"] and configuration["removeAttributes"].
                        element_attributes.difference(
                            configuration
                                .removeAttributes
                                .as_deref()
                                .unwrap_or_default(),
                        );

                        element.set_attributes(Some(element_attributes));
                    }

                    // Step 4.4.2. If element["removeAttributes"] exists:
                    if let Some(mut element_remove_attributes) = element.remove_attributes_mut() {
                        // Step 4.4.2.1. Set element["removeAttributes"] to remove duplicates from
                        // element["removeAttributes"].
                        element_remove_attributes = element_remove_attributes.remove_duplicates();

                        // Step 4.4.2.2. Set element["removeAttributes"] to the difference of
                        // element["removeAttributes"] and configuration["removeAttributes"].
                        element_remove_attributes.difference(
                            configuration
                                .removeAttributes
                                .as_deref()
                                .unwrap_or_default(),
                        );
                    }
                },
            }

            // Step 4.5. If configuration["elements"] does not contain element:
            let configuration_elements = configuration
                .elements
                .as_mut()
                .expect("Guaranteed by Step 4");
            if !configuration_elements.contains_item(&element) {
                // Step 4.5.1. Comment: This is the case with a global allow-list that does not yet
                // contain element.

                // Step 4.5.2. Append element to configuration["elements"].
                configuration_elements.push(element.clone());

                // Step 4.5.3. Return true.
                return true;
            }

            // Step 4.6. Comment: This is the case with a global allow-list that already contains
            // element.

            // Step 4.7. Let current element be the item in configuration["elements"] where
            // item["name"] equals element["name"] and item["namespace"] equals
            // element["namespace"].
            let current_element = configuration_elements
                .iter()
                .find(|item| {
                    item.name() == element.name() && item.namespace() == element.namespace()
                })
                .expect("Guaranteed by Step 4.5 and Step 4.5.2");

            // Step 4.8. If element equals current element then return modified.
            if element == *current_element {
                return modified;
            }

            // Step 4.9. Remove element from configuration["elements"].
            configuration_elements.remove_item(&element);

            // Step 4.10. Append element to configuration["elements"]
            configuration_elements.push(element);

            // Step 4.11. Return true.
            true
        }
        // Step 5. Otherwise:
        else {
            // Step 5.1. If element["attributes"] exists or element["removeAttributes"] with default
            // « » is not empty:
            if element.attributes().is_some() ||
                !element.remove_attributes().unwrap_or_default().is_empty()
            {
                // Step 5.1.1. The user agent may report a warning to the console that this
                // operation is not supported.
                Console::internal_warn(
                    &self.global(),
                    "Do not support adding an element with attributes to a sanitizer \
                        whose configuration[\"elements\"] does not exist."
                        .into(),
                );

                // Step 5.1.2. Return false.
                return false;
            }

            // Step 5.2. Set modified to the result of remove element from
            // configuration["replaceWithChildrenElements"].
            let modified = if let Some(replace_with_children_elements) =
                &mut configuration.replaceWithChildrenElements
            {
                replace_with_children_elements.remove_item(&element)
            } else {
                false
            };

            // Step 5.3. If configuration["removeElements"] does not contain element:
            if !configuration
                .removeElements
                .as_ref()
                .is_some_and(|configuration_remove_elements| {
                    configuration_remove_elements.contains_item(&element)
                })
            {
                // Step 5.3.1. Comment: This is the case with a global remove-list that does not
                // contain element.

                // Step 5.3.2. Return modified.
                return modified;
            }

            // Step 5.4. Comment: This is the case with a global remove-list that contains element.

            // Step 5.5. Remove element from configuration["removeElements"].
            if let Some(configuration_remove_elements) = &mut configuration.removeElements {
                configuration_remove_elements.remove_item(&element);
            }

            // Step 5.6. Return true.
            true
        }
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-removeelement>
    fn RemoveElement(&self, element: SanitizerElement) -> bool {
        // Remove an element with element and this’s configuration.
        self.configuration.borrow_mut().remove_element(element)
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-replaceelementwithchildren>
    fn ReplaceElementWithChildren(&self, element: SanitizerElement) -> bool {
        // Step 1. Let configuration be this’s configuration.
        let mut configuration = self.configuration.borrow_mut();

        // Step 2. Assert: configuration is valid.
        debug_assert!(configuration.is_valid());

        // Step 3. Set element to the result of canonicalize a sanitizer element with element.
        let element = element.canonicalize();

        // Step 4. If the built-in non-replaceable elements list contains element:
        if built_in_non_replaceable_elements_list().contains_item(&element) {
            // Step 4.1. Return false.
            return false;
        }

        // Step 5. If configuration["replaceWithChildrenElements"] contains element:
        if configuration
            .replaceWithChildrenElements
            .as_ref()
            .is_some_and(|configuration_replace_with_children_elements| {
                configuration_replace_with_children_elements.contains_item(&element)
            })
        {
            // Step 5.1. Return false.
            return false;
        }

        // Step 6. Remove element from configuration["removeElements"].
        if let Some(configuration_remove_elements) = &mut configuration.removeElements {
            configuration_remove_elements.remove_item(&element);
        }

        // Step 7. Remove element from configuration["elements"] list.
        if let Some(configuration_elements) = &mut configuration.elements {
            configuration_elements.remove_item(&element);
        }

        // Step 8. Add element to configuration["replaceWithChildrenElements"].
        if let Some(configuration_replace_with_children_elements) =
            &mut configuration.replaceWithChildrenElements
        {
            configuration_replace_with_children_elements.add_item(element);
        } else {
            configuration.replaceWithChildrenElements = Some(vec![element]);
        }

        // Step 9. Return true.
        true
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-allowattribute>
    fn AllowAttribute(&self, attribute: SanitizerAttribute) -> bool {
        // Step 1. Let configuration be this’s configuration.
        let mut configuration = self.configuration.borrow_mut();

        // Step 2. Assert: configuration is valid.
        debug_assert!(configuration.is_valid());

        // Step 3. Set attribute to the result of canonicalize a sanitizer attribute with attribute.
        let attribute = attribute.canonicalize();

        // Step 4. If configuration["attributes"] exists:
        if configuration.attributes.is_some() {
            // Step 4.1. Comment: If we have a global allow-list, we need to add attribute.

            // Step 4.2. If configuration["dataAttributes"] is true and attribute is a custom data
            // attribute, then return false.
            if configuration.dataAttributes == Some(true) &&
                is_custom_data_attribute(
                    &attribute.name().str(),
                    attribute
                        .namespace()
                        .map(|namespace| namespace.str())
                        .as_deref(),
                )
            {
                return false;
            }

            // Step 4.3. If configuration["attributes"] contains attribute return false.
            if configuration
                .attributes
                .as_ref()
                .is_some_and(|configuration_attributes| {
                    configuration_attributes.contains(&attribute)
                })
            {
                return false;
            }

            // Step 4.4. Comment: Fix-up per-element allow and remove lists.

            // Step 4.5. If configuration["elements"] exists:
            if let Some(configuration_elements) = &mut configuration.elements {
                // Step 4.5.1. For each element in configuration["elements"]:
                for element in configuration_elements.iter_mut() {
                    // Step 4.5.1.1. If element["attributes"] with default « » contains attribute:
                    // Step 4.5.1.1.1. Remove attribute from element["attributes"].
                    if let Some(element_attributes) = element.attributes_mut() {
                        element_attributes
                            .retain(|element_attribute| *element_attribute != attribute);
                    }

                    // Step 4.5.1.2. Assert: element["removeAttributes"] with default « » does not
                    // contain attribute.
                    debug_assert!(!element.remove_attributes().is_some_and(
                        |element_remove_attributes| element_remove_attributes.contains(&attribute)
                    ));
                }
            }

            // Step 4.6. Append attribute to configuration["attributes"]
            if let Some(configuration_attributes) = &mut configuration.attributes {
                configuration_attributes.push(attribute);
            } else {
                configuration.attributes = Some(vec![attribute]);
            }

            // Step 4.7. Return true.
            true
        }
        // Step 5. Otherwise:
        else {
            // Step 5.1. Comment: If we have a global remove-list, we need to remove attribute.

            // Step 5.2. If configuration["removeAttributes"] does not contain attribute:
            if !configuration.removeAttributes.as_ref().is_some_and(
                |configuration_remove_attributes| {
                    configuration_remove_attributes.contains(&attribute)
                },
            ) {
                // Step 5.2.1. Return false.
                return false;
            }

            // Step 5.3. Remove attribute from configuration["removeAttributes"].
            if let Some(configuration_remove_attributes) = &mut configuration.removeAttributes {
                configuration_remove_attributes.retain(|configuration_remove_attribute| {
                    *configuration_remove_attribute != attribute
                });
            }

            // Step 5.4. Return true.
            true
        }
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-removeattribute>
    fn RemoveAttribute(&self, attribute: SanitizerAttribute) -> bool {
        // Remove an attribute with attribute and this’s configuration.
        self.configuration.borrow_mut().remove_attribute(attribute)
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-setcomments>
    fn SetComments(&self, allow: bool) -> bool {
        // Step 1. Let configuration be this’s configuration.
        let mut configuration = self.configuration.borrow_mut();

        // Step 2. Assert: configuration is valid.
        debug_assert!(configuration.is_valid());

        // Step 3. If configuration["comments"] exists and configuration["comments"] equals allow,
        // then return false;
        if configuration
            .comments
            .is_some_and(|configuration_comments| configuration_comments == allow)
        {
            return false;
        }

        // Step 4. Set configuration["comments"] to allow.
        configuration.comments = Some(allow);

        // Step 5. Return true.
        true
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-setdataattributes>
    fn SetDataAttributes(&self, allow: bool) -> bool {
        // Step 1. Let configuration be this’s configuration.
        let mut configuration = self.configuration.borrow_mut();

        // Step 2. Assert: configuration is valid.
        debug_assert!(configuration.is_valid());

        // Step 3. If configuration["attributes"] does not exist, then return false.
        if configuration.attributes.is_none() {
            return false;
        }

        // Step 4. If configuration["dataAttributes"] equals allow, then return false.
        if configuration.dataAttributes == Some(allow) {
            return false;
        }

        // Step 5. If allow is true:
        if allow {
            // Step 5.1. Remove any items attr from configuration["attributes"] where attr is a
            // custom data attribute.
            if let Some(configuration_attributes) = &mut configuration.attributes {
                configuration_attributes.retain(|attribute| {
                    !is_custom_data_attribute(
                        &attribute.name().str(),
                        attribute
                            .namespace()
                            .map(|namespace| namespace.str())
                            .as_deref(),
                    )
                });
            }

            // Step 5.2. If configuration["elements"] exists:
            if let Some(configuration_elements) = &mut configuration.elements {
                // Step 5.2.1. For each element in configuration["elements"]:
                for element in configuration_elements {
                    // Step 5.2.1.1. If element["attributes"] exists:
                    if let Some(element_attributes) = element.attributes_mut() {
                        // Step 5.2.1.1.1. Remove any items attr from element["attributes"] where
                        // attr is a custom data attribute.
                        element_attributes.retain(|attribute| {
                            !is_custom_data_attribute(
                                &attribute.name().str(),
                                attribute
                                    .namespace()
                                    .map(|namespace| namespace.str())
                                    .as_deref(),
                            )
                        });
                    }
                }
            }
        }

        // Step 6. Set configuration["dataAttributes"] to allow.
        configuration.dataAttributes = Some(allow);

        // Step 7. Return true.
        true
    }

    /// <https://wicg.github.io/sanitizer-api/#dom-sanitizer-removeunsafe>
    fn RemoveUnsafe(&self) -> bool {
        // Update this’s configuration with the result of calling remove unsafe on this’s
        // configuration.
        self.configuration.borrow_mut().remove_unsafe()
    }
}

trait SanitizerConfigAlgorithm {
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-valid>
    fn is_valid(&self) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-remove-an-element>
    fn remove_element(&mut self, element: SanitizerElement) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-remove-an-attribute>
    fn remove_attribute(&mut self, attribute: SanitizerAttribute) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-remove-unsafe>
    fn remove_unsafe(&mut self) -> bool;

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
                if built_in_non_replaceable_elements_list().contains_item(element) {
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
                            .all(|entry| config_attributes.contains_item(entry))
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

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-remove-an-element>
    fn remove_element(&mut self, element: SanitizerElement) -> bool {
        // Step 1. Assert: configuration is valid.
        debug_assert!(self.is_valid());

        // Step 2. Set element to the result of canonicalize a sanitizer element with element.
        let element = element.canonicalize();

        // Step 3. Set modified to the result of remove element from
        // configuration["replaceWithChildrenElements"].
        let modified = if let Some(configuration_replace_with_children_elements) =
            &mut self.replaceWithChildrenElements
        {
            configuration_replace_with_children_elements.remove_item(&element)
        } else {
            false
        };

        // Step 4. If configuration["elements"] exists:
        if let Some(configuration_elements) = &mut self.elements {
            // Step 4.1. If configuration["elements"] contains element:
            if configuration_elements.contains_item(&element) {
                // Step 4.1.1. Comment: We have a global allow list and it contains element.

                // Step 4.1.2. Remove element from configuration["elements"].
                configuration_elements.remove_item(&element);

                // Step 4.1.3. Return true.
                return true;
            }

            // Step 4.2. Comment: We have a global allow list and it does not contain element.

            // Step 4.3. Return modified.
            modified
        }
        // Step 5. Otherwise:
        else {
            // Step 5.1. If configuration["removeElements"] contains element:
            if self
                .removeElements
                .as_mut()
                .is_some_and(|configuration_remove_elements| {
                    configuration_remove_elements.contains_item(&element)
                })
            {
                // Step 5.1.1. Comment: We have a global remove list and it already contains element.

                // Step 5.1.2. Return modified.
                return modified;
            }

            // Step 5.2. Comment: We have a global remove list and it does not contain element.

            // Step 5.3. Add element to configuration["removeElements"].
            if let Some(configuration_remove_elements) = &mut self.removeElements {
                configuration_remove_elements.add_item(element);
            } else {
                self.removeElements = Some(vec![element]);
            }

            // Step 5.4. Return true.
            true
        }
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizer-remove-an-attribute>
    fn remove_attribute(&mut self, attribute: SanitizerAttribute) -> bool {
        // Step 1. Assert: configuration is valid.
        debug_assert!(self.is_valid());

        // Step 2. Set attribute to the result of canonicalize a sanitizer attribute with attribute.
        let attribute = attribute.canonicalize();

        // Step 3. If configuration["attributes"] exists:
        if self.attributes.is_some() {
            // Step 3.1. Comment: If we have a global allow-list, we need to remove attribute.

            // Step 3.2. Set modified to the result of remove attribute from
            // configuration["attributes"].
            let mut modified = self
                .attributes
                .as_mut()
                .is_some_and(|configuration_attributes| {
                    configuration_attributes.remove_item(&attribute)
                });

            // Step 3.3. Comment: Fix-up per-element allow and remove lists.

            // Step 3.4. If configuration["elements"] exists:
            if let Some(configuration_elements) = &mut self.elements {
                // Step 3.4.1. For each element of configuration["elements"]:
                for element in configuration_elements {
                    // Step 3.4.1.1. If element["attributes"] with default « » contains attribute:
                    if element
                        .attributes()
                        .unwrap_or_default()
                        .contains(&attribute)
                    {
                        // Step 3.4.1.1.1. Set modified to true.
                        modified = true;

                        // Step 3.4.1.1.2. Remove attribute from element["attributes"].
                        if let Some(element_attributes) = element.attributes_mut() {
                            element_attributes
                                .retain(|element_attribute| *element_attribute != attribute);
                        }
                    }

                    // Step 3.4.1.2. If element["removeAttributes"] with default « » contains
                    // attribute:
                    if element
                        .remove_attributes()
                        .unwrap_or_default()
                        .contains(&attribute)
                    {
                        // Step 3.4.1.2.1. Assert: modified is true.
                        assert!(modified);

                        // Step 3.4.1.2.2. Remove attribute from element["removeAttributes"].
                        if let Some(element_remove_attributes) = element.remove_attributes_mut() {
                            element_remove_attributes.retain(|element_remove_attribute| {
                                *element_remove_attribute != attribute
                            });
                        }
                    }
                }
            }

            // Step 3.5. Return modified.
            modified
        }
        // Step 4. Otherwise:
        else {
            // Step 4.1. Comment: If we have a global remove-list, we need to add attribute.

            // Step 4.2. If configuration["removeAttributes"] contains attribute return false.
            if self
                .removeAttributes
                .as_ref()
                .is_some_and(|configuration_remove_attributes| {
                    configuration_remove_attributes.contains(&attribute)
                })
            {
                return false;
            }

            // Step 4.3. Comment: Fix-up per-element allow and remove lists.

            // Step 4.4. If configuration["elements"] exists:
            if let Some(configuration_elements) = &mut self.elements {
                // Step 4.4.1. For each element in configuration["elements"]:
                for element in configuration_elements {
                    // Step 4.4.1.1. If element["attributes"] with default « » contains attribute:
                    // Step 4.4.1.1.1. Remove attribute from element["attributes"].
                    if let Some(element_attributes) = element.attributes_mut() {
                        element_attributes
                            .retain(|element_attribute| *element_attribute != attribute);
                    }

                    // Step 4.4.1.2. If element["removeAttributes"] with default « » contains
                    // attribute:
                    // Step 4.4.1.2.1. Remove attribute from element["removeAttributes"].
                    if let Some(element_remove_attributes) = element.remove_attributes_mut() {
                        element_remove_attributes.retain(|element_remove_attribute| {
                            *element_remove_attribute != attribute
                        });
                    }
                }
            }

            // Step 4.5. Append attribute to configuration["removeAttributes"]
            if let Some(configuration_remove_attributes) = &mut self.removeAttributes {
                configuration_remove_attributes.push(attribute);
            } else {
                self.removeAttributes = Some(vec![attribute]);
            }

            // Step 4.6. Return true.
            true
        }
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-remove-unsafe>
    fn remove_unsafe(&mut self) -> bool {
        // Step 1. Assert: The key set of built-in safe baseline configuration equals « [
        // "removeElements", "removeAttributes" ] ».
        let baseline = built_in_safe_baseline_configuration();
        assert!(baseline.removeElements.is_some() && baseline.removeAttributes.is_some());

        // Step 2. Assert: configuration is valid.
        debug_assert!(self.is_valid());

        // Step 3. Let result be false.
        let mut result = false;

        // Step 4. For each element in built-in safe baseline configuration["removeElements"]:
        for element in baseline.removeElements.unwrap_or_default() {
            // Step 4.1. Call remove an element element from configuration.
            // Step 4.2. If the call returned true, set result to true.
            if self.remove_element(element) {
                result = true;
            }
        }

        // Step 5. For each attribute in built-in safe baseline configuration["removeAttributes"]:
        for attribute in baseline.removeAttributes.unwrap_or_default() {
            // Step 5.1. Call remove an attribute attribute from configuration.
            // Step 5.2. If the call returned true, set result to true.
            if self.remove_attribute(attribute) {
                result = true;
            }
        }

        // Step 6. For each attribute listed in event handler content attributes:
        for attribute in CONTENT_EVENT_HANDLER_NAMES.iter() {
            // Step 6.1. Call remove an attribute attribute from configuration.
            // Step 6.2. If the call returned true, set result to true.
            let attribute = SanitizerAttribute::String(DOMString::from(*attribute));
            if self.remove_attribute(attribute) {
                result = true;
            }
        }

        // Step 7. Return result.
        result
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
    fn contains_item<S: NameMember>(&self, other: &S) -> bool;

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
    fn contains_item<S: NameMember>(&self, other: &S) -> bool {
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

/// Supporting algorithms on lists of elements and lists of attributes, from the specification.
trait NameVec<T>
where
    T: NameMember + Canonicalization + Clone,
{
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-remove>
    fn remove_item<S: NameMember>(&mut self, item: &S) -> bool;

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-add>
    fn add_item(&mut self, name: T);

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-remove-duplicates>
    fn remove_duplicates(&mut self) -> &mut Self;

    /// Set itself to the set intersection of itself and another list.
    ///
    /// <https://infra.spec.whatwg.org/#set-intersection>
    fn intersection<S>(&mut self, others: &[S])
    where
        S: NameMember + Canonicalization + Clone;

    /// <https://infra.spec.whatwg.org/#set-difference>
    fn difference(&mut self, others: &[T]);
}

impl<T> NameVec<T> for Vec<T>
where
    T: NameMember + Canonicalization + Clone,
{
    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-remove>
    fn remove_item<S: NameMember>(&mut self, item: &S) -> bool {
        // Step 1. Set removed to false.
        let mut removed = false;

        // Step 2. For each entry of list:
        // Step 2.1. If item["name"] equals entry["name"] and item["namespace"] equals entry["namespace"]:
        // Step 2.1.1. Remove item entry from list.
        // Step 2.1.2. Set removed to true.
        self.retain(|entry| {
            let matched = item.name() == entry.name() && item.namespace() == entry.namespace();
            if matched {
                removed = true;
            }
            !matched
        });

        // Step 3. Return removed.
        removed
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-add>
    fn add_item(&mut self, name: T) {
        // Step 1. If list contains name, then return.
        if self.contains_item(&name) {
            return;
        };

        // Step 2. Append name to list.
        self.push(name);
    }

    /// <https://wicg.github.io/sanitizer-api/#sanitizerconfig-remove-duplicates>
    fn remove_duplicates(&mut self) -> &mut Self {
        // Step 1. Let result be « ».
        // Step 2. For each entry of list, add entry to result.
        // Step 3. Return result.
        self.sort_by(|item_a, item_b| item_a.compare(item_b));
        self.dedup_by_key(|item| (item.name().clone(), item.namespace().cloned()));
        self
    }

    /// Set itself to the set intersection of itself and another list.
    ///
    /// <https://infra.spec.whatwg.org/#set-intersection>
    fn intersection<S>(&mut self, others: &[S])
    where
        S: NameMember + Canonicalization + Clone,
    {
        // The intersection of ordered sets A and B, is the result of creating a new ordered set set
        // and, for each item of A, if B contains item, appending item to set.
        self.retain(|item| {
            others
                .iter()
                .any(|other| other.name() == item.name() && other.namespace() == item.namespace())
        })
    }

    /// Set itself to the set difference of itself and another list.
    ///
    /// <https://infra.spec.whatwg.org/#set-difference>
    fn difference(&mut self, others: &[T]) {
        // The difference of ordered sets A and B, is the result of creating a new ordered set set
        // and, for each item of A, if B does not contain item, appending item to set.
        self.retain(|item| {
            !others
                .iter()
                .any(|other| other.name() == item.name() && other.namespace() == item.namespace())
        })
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

    // <https://wicg.github.io/sanitizer-api/#sanitizerconfig-less-than-item>
    fn is_less_than_item(&self, item_b: &Self) -> bool {
        let item_a = self;
        match item_a.namespace() {
            // Step 1. If itemA["namespace"] is null:
            None => {
                // Step 1.1. If itemB["namespace"] is not null, then return true.
                if item_b.namespace().is_some() {
                    return true;
                }
            },
            // Step 2. Otherwise:
            Some(item_a_namespace) => {
                // Step 2.1. If itemB["namespace"] is null, then return false.
                if item_b.namespace().is_none() {
                    return false;
                }

                // Step 2.2. If itemA["namespace"] is code unit less than itemB["namespace"], then
                // return true.
                if item_b
                    .namespace()
                    .is_some_and(|item_b_namespace| item_a_namespace < item_b_namespace)
                {
                    return true;
                }

                // Step 2.3. If itemA["namespace"] is not itemB["namespace"], then return false.
                if item_b
                    .namespace()
                    .is_some_and(|item_b_namespace| item_a_namespace != item_b_namespace)
                {
                    return false;
                }
            },
        }

        // Step 3. Return itemA["name"] is code unit less than itemB["name"].
        item_a.name() < item_b.name()
    }

    /// Wrapper of [`NameMember::is_less_than_item`] that returns [`std::cmp::Ordering`].
    fn compare(&self, other: &Self) -> Ordering {
        if self.is_less_than_item(other) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
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
    fn attributes_mut(&mut self) -> Option<&mut Vec<SanitizerAttribute>>;
    fn remove_attributes(&self) -> Option<&[SanitizerAttribute]>;
    fn remove_attributes_mut(&mut self) -> Option<&mut Vec<SanitizerAttribute>>;

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

    fn attributes_mut(&mut self) -> Option<&mut Vec<SanitizerAttribute>> {
        match self {
            SanitizerElementWithAttributes::String(_) => None,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.attributes.as_mut()
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

    fn remove_attributes_mut(&mut self) -> Option<&mut Vec<SanitizerAttribute>> {
        match self {
            SanitizerElementWithAttributes::String(_) => None,
            SanitizerElementWithAttributes::SanitizerElementNamespaceWithAttributes(dictionary) => {
                dictionary.removeAttributes.as_mut()
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

/// <https://wicg.github.io/sanitizer-api/#built-in-safe-baseline-configuration>
fn built_in_safe_baseline_configuration() -> SanitizerConfig {
    const REMOVE_ELEMENTS: &[(&str, &Namespace)] = &[
        ("embed", &ns!(html)),
        ("frame", &ns!(html)),
        ("iframe", &ns!(html)),
        ("object", &ns!(html)),
        ("script", &ns!(html)),
        ("script", &ns!(svg)),
        ("use", &ns!(svg)),
    ];

    let remove_elements = REMOVE_ELEMENTS
        .iter()
        .map(|&(name, namespace)| {
            SanitizerElement::SanitizerElementNamespace(SanitizerElementNamespace {
                name: name.into(),
                namespace: Some(namespace.to_string().into()),
            })
        })
        .collect();

    SanitizerConfig {
        elements: None,
        removeElements: Some(remove_elements),
        replaceWithChildrenElements: None,
        attributes: None,
        removeAttributes: Some(Vec::new()),
        comments: None,
        dataAttributes: None,
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
            name: "math".into(),
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
