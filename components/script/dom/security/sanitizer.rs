/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::SanitizerBinding::{
    SanitizerConfig, SanitizerMethods, SanitizerPresets,
};
use crate::dom::bindings::codegen::UnionTypes::SanitizerConfigOrSanitizerPresets;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use crate::dom::bindings::root::DomRoot;
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
        configuration: SanitizerConfig,
        _allow_comments_and_data_attributes: bool,
    ) -> bool {
        // TODO:
        // Step 1. Canonicalize configuration with allowCommentsAndDataAttributes.

        // TODO:
        // Step 2. If configuration is not valid, then return false.

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

                // TODO:
                // Step 1.2. Set configuration to the built-in safe default configuration.
                SanitizerConfig::default()
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
