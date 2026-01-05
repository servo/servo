/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel;
use base::id::BrowsingContextId;
use embedder_traits::WebDriverScriptCommand;
use serde_json::Value;
use webdriver::command::JavascriptCommandParameters;
use webdriver::common::{ELEMENT_KEY, FRAME_KEY, SHADOW_KEY, WINDOW_KEY};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};

use crate::{Handler, VerifyBrowsingContextIsOpen, wait_for_ipc_response_flatten};

impl Handler {
    /// <https://w3c.github.io/webdriver/#dfn-extract-the-script-arguments-from-a-request>
    pub(crate) fn extract_script_arguments(
        &self,
        parameters: JavascriptCommandParameters,
    ) -> WebDriverResult<(String, Vec<String>)> {
        // Step 1. Let script be the result of getting a property named "script" from parameters
        // Step 2. (Done) If script is not a String, return error with error code invalid argument.
        let script = parameters.script;

        // Step 3. Let args be the result of getting a property named "args" from parameters.
        // Step 4. (Done) If args is not an Array return error with error code invalid argument.
        // Step 5. Let `arguments` be JSON deserialize with session and args.
        let args: Vec<String> = parameters
            .args
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|value| self.json_deserialize(value))
            .collect::<WebDriverResult<Vec<_>>>()?;

        Ok((script, args))
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-web-element>
    fn deserialize_web_element(&self, element: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the web element identifier property from object.
        let element_ref = match element {
            Value::String(string) => string.clone(),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument, "")),
        };

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::GetKnownElement(element_ref.clone(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        wait_for_ipc_response_flatten(receiver)?;
        // Step 4. Return success with data element.
        Ok(format!("window.webdriverElement(\"{}\")", element_ref))
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-shadow-root>
    fn deserialize_shadow_root(&self, shadow_root: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the shadow root identifier property from object.
        let shadow_root_ref = match shadow_root {
            Value::String(string) => string.clone(),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument, "")),
        };

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::GetKnownShadowRoot(shadow_root_ref.clone(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        wait_for_ipc_response_flatten(receiver)?;
        // Step 4. Return success with data element.
        Ok(format!(
            "window.webdriverShadowRoot(\"{}\")",
            shadow_root_ref
        ))
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-web-frame>
    fn deserialize_web_frame(&self, frame: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the web frame identifier property from object.
        let frame_ref = match frame {
            Value::String(string) => string.clone(),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument, "")),
        };

        // Step 3. Let browsing context be the browsing context whose window handle is reference,
        // or null if no such browsing context exists.
        let Some(browsing_context_id) = BrowsingContextId::from_string(&frame_ref) else {
            // Step 4. If browsing context is null or a top-level browsing context,
            // return error with error code no such frame.
            return Err(WebDriverError::new(ErrorStatus::NoSuchFrame, ""));
        };

        match self.verify_browsing_context_is_open(browsing_context_id) {
            // Step 5. Return success with data browsing context's associated window.
            Ok(_) => Ok(format!("window.webdriverFrame(\"{frame_ref}\")")),
            // Part of Step 4.
            Err(_) => Err(WebDriverError::new(ErrorStatus::NoSuchFrame, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-web-window>
    fn deserialize_web_window(&self, window: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the web window identifier property from object.
        let window_ref = match window {
            Value::String(string) => string.clone(),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument, "")),
        };

        // Step 3. Let browsing context be the browsing context whose window handle is reference,
        // or null if no such browsing context exists.
        let (sender, receiver) = generic_channel::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::GetKnownWindow(window_ref.clone(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        // Step 4. If browsing context is null or not a top-level browsing context,
        // return error with error code no such window.
        wait_for_ipc_response_flatten(receiver)?;
        // Step 5. Return success with data browsing context's associated window.
        Ok(format!("window.webdriverWindow(\"{window_ref}\")"))
    }

    /// <https://w3c.github.io/webdriver/#dfn-json-deserialize>
    fn json_deserialize(&self, v: &Value) -> WebDriverResult<String> {
        let res = match v {
            Value::Array(list) => {
                let elems = list
                    .iter()
                    .map(|v| self.json_deserialize(v))
                    .collect::<WebDriverResult<Vec<_>>>()?;
                format!("[{}]", elems.join(", "))
            },
            Value::Object(map) => {
                if let Some(id) = map.get(ELEMENT_KEY) {
                    return self.deserialize_web_element(id);
                }
                if let Some(id) = map.get(SHADOW_KEY) {
                    return self.deserialize_shadow_root(id);
                }
                if let Some(id) = map.get(FRAME_KEY) {
                    return self.deserialize_web_frame(id);
                }
                if let Some(id) = map.get(WINDOW_KEY) {
                    return self.deserialize_web_window(id);
                }
                let elems = map
                    .iter()
                    .map(|(k, v)| {
                        let key = serde_json::to_string(k)?;
                        let arg = self.json_deserialize(v)?;
                        Ok(format!("{key}: {arg}"))
                    })
                    .collect::<WebDriverResult<Vec<String>>>()?;
                format!("{{{}}}", elems.join(", "))
            },
            _ => serde_json::to_string(v)?,
        };

        Ok(res)
    }
}
