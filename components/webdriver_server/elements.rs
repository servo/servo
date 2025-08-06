/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{WebDriverCommandMsg, WebDriverScriptCommand};
use ipc_channel::ipc;
use serde_json::Value;
use webdriver::command::JavascriptCommandParameters;
use webdriver::error::{WebDriverError, WebDriverResult};

use crate::{Handler, wait_for_script_response};

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
        let element_ref: String = match element {
            Value::String(s) => s.clone(),
            _ => element.to_string(),
        };

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let browsing_context_id = self.session()?.browsing_context_id;
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            WebDriverScriptCommand::GetKnownElement(element_ref.clone(), sender),
        ))?;

        match wait_for_script_response(receiver)? {
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
            _ => shadow_root.to_string(),
        };

        // Step 3. Let element be the result of trying to get a known element with session and reference.
        let browsing_context_id = self.session()?.browsing_context_id;
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            WebDriverScriptCommand::GetKnownShadowRoot(shadow_root_ref.clone(), sender),
        ))?;

        match wait_for_script_response(receiver)? {
            // Step 4. Return success with data element.
            Ok(_) => Ok(format!(
                "window.webdriverShadowRoot(\"{}\")",
                shadow_root_ref
            )),
            Err(err) => Err(WebDriverError::new(err, "No such shadowroot")),
        }
    }

    /// This function is equivalent to step 5 of
    /// <https://w3c.github.io/webdriver/#dfn-extract-the-script-arguments-from-a-request>
    fn webdriver_value_to_js_argument(&self, v: &Value) -> WebDriverResult<String> {
        let res = match v {
            Value::String(s) => format!("\"{}\"", s),
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Array(list) => {
                let elems = list
                    .iter()
                    .map(|v| self.webdriver_value_to_js_argument(v))
                    .collect::<WebDriverResult<Vec<_>>>()?;
                format!("[{}]", elems.join(", "))
            },
            Value::Object(map) => {
                let key = map.keys().next().map(String::as_str);
                match (key, map.values().next()) {
                    (Some(ELEMENT_IDENTIFIER), Some(id)) => {
                        return self.deserialize_web_element(id);
                    },
                    (Some(FRAME_IDENTIFIER), Some(id)) => {
                        let frame_ref = match id {
                            Value::String(s) => s.clone(),
                            _ => id.to_string(),
                        };
                        return Ok(format!("window.webdriverFrame(\"{}\")", frame_ref));
                    },
                    (Some(WINDOW_IDENTIFIER), Some(id)) => {
                        let window_ref = match id {
                            Value::String(s) => s.clone(),
                            _ => id.to_string(),
                        };
                        return Ok(format!("window.webdriverWindow(\"{}\")", window_ref));
                    },
                    (Some(SHADOW_ROOT_IDENTIFIER), Some(id)) => {
                        return self.deserialize_shadow_root(id);
                    },
                    _ => {},
                }
                let elems = map
                    .iter()
                    .map(|(k, v)| {
                        let arg = self.webdriver_value_to_js_argument(v)?;
                        Ok(format!("{}: {}", k, arg))
                    })
                    .collect::<WebDriverResult<Vec<String>>>()?;
                format!("{{{}}}", elems.join(", "))
            },
        };

        Ok(res)
    }
}
