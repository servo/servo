/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::Duration;

use serde_json::Value;
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};

/// Initial script timeout from
/// <https://w3c.github.io/webdriver/#dfn-timeouts-configuration>.
pub(crate) const DEFAULT_SCRIPT_TIMEOUT: u64 = 30_000;

/// Initial page load timeout from
/// <https://w3c.github.io/webdriver/#dfn-timeouts-configuration>.
pub(crate) const DEFAULT_PAGE_LOAD_TIMEOUT: u64 = 300_000;

/// Initial initial wait timeout from
/// <https://w3c.github.io/webdriver/#dfn-timeouts-configuration>.
pub(crate) const DEFAULT_IMPLICIT_WAIT: u64 = 0;

/// An amount of time to wait before considering that a screenshot has timed out.
/// If after 10 seconds the screenshot cannot be taken, assume that the test has
/// timed out.
pub(crate) const SCREENSHOT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct TimeoutsConfiguration {
    pub(crate) script: Option<u64>,
    pub(crate) page_load: Option<u64>,
    pub(crate) implicit_wait: Option<u64>,
    pub(crate) sleep_interval: u64,
}

impl Default for TimeoutsConfiguration {
    fn default() -> Self {
        TimeoutsConfiguration {
            script: Some(DEFAULT_SCRIPT_TIMEOUT),
            page_load: Some(DEFAULT_PAGE_LOAD_TIMEOUT),
            implicit_wait: Some(DEFAULT_IMPLICIT_WAIT),
            sleep_interval: 10,
        }
    }
}

/// <https://w3c.github.io/webdriver/#dfn-deserialize-as-timeouts-configuration>
pub(crate) fn deserialize_as_timeouts_configuration(
    timeouts: &Value,
) -> WebDriverResult<TimeoutsConfiguration> {
    if let Value::Object(map) = timeouts {
        let mut config = TimeoutsConfiguration::default();

        // Step 3: For each key → value in timeouts:
        for (key, value) in map {
            // Step 3.1: If «"script", "pageLoad", "implicit"» does not contain key, then continue.
            let target = match key.as_str() {
                "implicit" => &mut config.implicit_wait,
                "pageLoad" => &mut config.page_load,
                "script" => &mut config.script,
                _ => continue,
            };

            // Step 3.2: If value is neither null nor a number greater than or equal to 0
            // and less than or equal to the maximum safe integer return error with error
            // code invalid argument.
            // Step 3.3: Run the substeps matching key:
            //  - "script": Set configuration's script timeout to value.
            //  - "pageLoad": Set configuration's page load timeout to value.
            //  - "implicit": Set configuration's implicit wait timeout to value.
            *target = match value {
                Value::Null => None,
                _ => Some(value.as_f64().ok_or_else(|| {
                    WebDriverError::new(ErrorStatus::InvalidArgument, "Invalid value for {key}")
                })? as u64),
            };
        }

        Ok(config)
    } else {
        Err(WebDriverError::new(
            ErrorStatus::InvalidArgument,
            "Expected an object for timeouts",
        ))
    }
}

pub(crate) fn serialize_timeouts_configuration(timeouts: &TimeoutsConfiguration) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("script".to_string(), Value::from(timeouts.script));
    map.insert("pageLoad".to_string(), Value::from(timeouts.page_load));
    map.insert("implicit".to_string(), Value::from(timeouts.implicit_wait));
    Value::Object(map)
}
