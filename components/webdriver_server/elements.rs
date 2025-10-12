/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::WebDriverScriptCommand;
use ipc_channel::ipc;
use serde_json::Value;
use webdriver::command::JavascriptCommandParameters;
use webdriver::error::{WebDriverError, WebDriverResult};

use crate::{Handler, VerifyBrowsingContextIsOpen, wait_for_ipc_response};

/// <https://w3c.github.io/webdriver/#dfn-web-element-identifier>
const ELEMENT_IDENTIFIER: &str = "element-6066-11e4-a52e-4f735466cecf";
/// <https://w3c.github.io/webdriver/#dfn-web-frame-identifier>
const FRAME_IDENTIFIER: &str = "frame-075b-4da1-b6ba-e579c2d3230a";
/// <https://w3c.github.io/webdriver/#dfn-web-window-identifier>
const WINDOW_IDENTIFIER: &str = "window-fcc6-11e5-b4f8-330a88ab9d7f";
/// <https://w3c.github.io/webdriver/#dfn-shadow-root-identifier>
const SHADOW_ROOT_IDENTIFIER: &str = "shadow-6066-11e4-a52e-4f735466cecf";

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
    pub(crate) fn deserialize_web_element(&self, element: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the web element identifier property from object.
        let element_ref = match element {
            Value::String(s) => s.clone(),
            _ => unreachable!(),
        };

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let (sender, receiver) = ipc::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::GetKnownElement(element_ref.clone(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        match wait_for_ipc_response(receiver)? {
            // Step 4. Return success with data element.
            Ok(_) => Ok(format!("window.webdriverElement(\"{}\")", element_ref)),
            Err(err) => Err(WebDriverError::new(err, "No such element")),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-deserialize-a-shadow-root>
    pub(crate) fn deserialize_shadow_root(&self, shadow_root: &Value) -> WebDriverResult<String> {
        // Step 2. Let reference be the result of getting the shadow root identifier property from object.
        let shadow_root_ref = match shadow_root {
            Value::String(s) => s.clone(),
            _ => unreachable!(),
        };

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let (sender, receiver) = ipc::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::GetKnownShadowRoot(shadow_root_ref.clone(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        match wait_for_ipc_response(receiver)? {
            // Step 4. Return success with data element.
            Ok(_) => Ok(format!(
                "window.webdriverShadowRoot(\"{}\")",
                shadow_root_ref
            )),
            Err(err) => Err(WebDriverError::new(err, "No such shadowroot")),
        }
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
                if let Some(id) = map.get(ELEMENT_IDENTIFIER) {
                    return self.deserialize_web_element(id);
                }
                if let Some(id) = map.get(SHADOW_ROOT_IDENTIFIER) {
                    return self.deserialize_shadow_root(id);
                }
                if let Some(id) = map.get(FRAME_IDENTIFIER) {
                    let frame_ref = match id {
                        Value::String(s) => s.clone(),
                        _ => id.to_string(),
                    };
                    return Ok(format!("window.webdriverFrame(\"{frame_ref}\")"));
                }
                if let Some(id) = map.get(WINDOW_IDENTIFIER) {
                    let window_ref = match id {
                        Value::String(s) => s.clone(),
                        _ => id.to_string(),
                    };
                    return Ok(format!("window.webdriverWindow(\"{window_ref}\")"));
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
