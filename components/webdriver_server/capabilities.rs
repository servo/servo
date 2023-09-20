/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::{Map, Value};
use webdriver::capabilities::{BrowserCapabilities, Capabilities};
use webdriver::error::{WebDriverError, WebDriverResult};

pub struct ServoCapabilities {
    pub browser_name: String,
    pub browser_version: String,
    pub platform_name: Option<String>,
    pub accept_insecure_certs: bool,
    pub set_window_rect: bool,
    pub strict_file_interactability: bool,
    pub accept_proxy: bool,
    pub accept_custom: bool,
}

impl ServoCapabilities {
    pub fn new() -> ServoCapabilities {
        ServoCapabilities {
            browser_name: "servo".to_string(),
            browser_version: "0.0.1".to_string(),
            platform_name: get_platform_name(),
            accept_insecure_certs: false,
            set_window_rect: true,
            strict_file_interactability: false,
            accept_proxy: false,
            accept_custom: false,
        }
    }
}

impl BrowserCapabilities for ServoCapabilities {
    fn init(&mut self, _: &Capabilities) {}

    fn browser_name(&mut self, _: &Capabilities) -> WebDriverResult<Option<String>> {
        Ok(Some(self.browser_name.clone()))
    }

    fn browser_version(&mut self, _: &Capabilities) -> WebDriverResult<Option<String>> {
        Ok(Some(self.browser_version.clone()))
    }

    fn compare_browser_version(&mut self, _: &str, _: &str) -> WebDriverResult<bool> {
        Ok(true)
    }

    fn platform_name(&mut self, _: &Capabilities) -> WebDriverResult<Option<String>> {
        Ok(self.platform_name.clone())
    }

    fn accept_insecure_certs(&mut self, _: &Capabilities) -> WebDriverResult<bool> {
        Ok(self.accept_insecure_certs)
    }

    fn set_window_rect(&mut self, _: &Capabilities) -> WebDriverResult<bool> {
        Ok(self.set_window_rect)
    }

    fn strict_file_interactability(&mut self, _: &Capabilities) -> WebDriverResult<bool> {
        Ok(self.strict_file_interactability)
    }

    fn accept_proxy(&mut self, _: &Map<String, Value>, _: &Capabilities) -> WebDriverResult<bool> {
        Ok(self.accept_proxy)
    }

    fn accept_custom(&mut self, _: &str, _: &Value, _: &Capabilities) -> WebDriverResult<bool> {
        Ok(self.accept_custom)
    }

    fn validate_custom(&mut self, _: &str, _: &Value) -> WebDriverResult<()> {
        Ok(())
    }

    fn web_socket_url(
        &mut self,
        _: &serde_json::Map<std::string::String, Value>,
    ) -> Result<bool, WebDriverError> {
        todo!()
    }

    fn webauthn_virtual_authenticators(
        &mut self,
        _: &serde_json::Map<std::string::String, Value>,
    ) -> Result<bool, WebDriverError> {
        todo!()
    }

    fn webauthn_extension_uvm(
        &mut self,
        _: &serde_json::Map<std::string::String, Value>,
    ) -> Result<bool, WebDriverError> {
        todo!()
    }

    fn webauthn_extension_prf(
        &mut self,
        _: &serde_json::Map<std::string::String, Value>,
    ) -> Result<bool, WebDriverError> {
        todo!()
    }

    fn webauthn_extension_large_blob(
        &mut self,
        _: &serde_json::Map<std::string::String, Value>,
    ) -> Result<bool, WebDriverError> {
        todo!()
    }

    fn webauthn_extension_cred_blob(
        &mut self,
        _: &serde_json::Map<std::string::String, Value>,
    ) -> Result<bool, WebDriverError> {
        todo!()
    }
}

fn get_platform_name() -> Option<String> {
    if cfg!(target_os = "windows") {
        Some("windows".to_string())
    } else if cfg!(target_os = "linux") {
        Some("linux".to_string())
    } else if cfg!(target_os = "macos") {
        Some("mac".to_string())
    } else {
        None
    }
}
