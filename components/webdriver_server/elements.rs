/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{WebDriverCommandMsg, WebDriverScriptCommand};
use ipc_channel::ipc;
use serde_json::Value;
use webdriver::command::JavascriptCommandParameters;
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};

use crate::{Handler, wait_for_script_response};

impl Handler {
    /// <https://w3c.github.io/webdriver/#dfn-extract-the-script-arguments-from-a-request>
    pub(crate) fn extract_script_arguments(
        &self,
        parameters: JavascriptCommandParameters,
    ) -> WebDriverResult<(String, Vec<String>)> {
        // Step 1. Let script be the result of getting a property named "script" from parameters
        // Step 2. (Skip) If script is not a String, return error with error code invalid argument.
        let script = parameters.script;

        // Step 3. Let args be the result of getting a property named "args" from parameters.
        // Step 4. (Skip) If args is not an Array return error with error code invalid argument.
        let args: Vec<String> = parameters
            .args
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|value| self.webdriver_value_to_js_argument(value))
            .collect::<WebDriverResult<Vec<_>>>()?;

        Ok((script, args))
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-web-element>
    pub(crate) fn deserialize_web_element(&self, element: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the web element identifier property from object.
        let element_ref = element.to_string();

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let browsing_context_id = self.session()?.browsing_context_id;
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            WebDriverScriptCommand::GetKnownElement(element_ref.clone(), sender),
        ))?;

        match wait_for_script_response(receiver)? {
            // Step 4. Return success with data element.
            Ok(_) => Ok(element_ref),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::NoSuchElement,
                "No such element",
            )),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-shadow-root>
    pub(crate) fn deserialize_shadow_root(&self, shadow_root: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the shadow root identifier property from object.
        let shadow_root_ref = shadow_root.to_string();

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let browsing_context_id = self.session()?.browsing_context_id;
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            WebDriverScriptCommand::GetKnownShadowRoot(shadow_root_ref.clone(), sender),
        ))?;

        match wait_for_script_response(receiver)? {
            // Step 4. Return success with data element.
            Ok(_) => Ok(shadow_root_ref),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::NoSuchShadowRoot,
                "No such shadow root",
            )),
        }
    }
}
