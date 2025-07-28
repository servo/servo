/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::Value;
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};

pub(crate) struct TimeoutsConfiguration {
    pub script: Option<u64>,
    pub page_load: u64,
    pub implicit_wait: u64,
}

impl Default for TimeoutsConfiguration {
    fn default() -> Self {
        TimeoutsConfiguration {
            script: Some(30_000),
            page_load: 300_000,
            implicit_wait: 0,
        }
    }
}

/// <https://w3c.github.io/webdriver/#dfn-deserialize-as-timeouts-configuration>
pub(crate) fn deserialize_as_timeouts_configuration(
    timeouts: &Value,
) -> WebDriverResult<TimeoutsConfiguration> {
    if let Value::Object(map) = timeouts {
        let mut config = TimeoutsConfiguration::default();
        for (key, value) in map {
            match key.as_str() {
                "implicit" => {
                    config.implicit_wait = value.as_f64().ok_or_else(|| {
                        WebDriverError::new(
                            ErrorStatus::InvalidArgument,
                            "Invalid implicit timeout",
                        )
                    })? as u64;
                },
                "pageLoad" => {
                    config.page_load = value.as_f64().ok_or_else(|| {
                        WebDriverError::new(
                            ErrorStatus::InvalidArgument,
                            "Invalid page load timeout",
                        )
                    })? as u64;
                },
                "script" => {
                    config.script = Some(value.as_f64().ok_or_else(|| {
                        WebDriverError::new(ErrorStatus::InvalidArgument, "Invalid script timeout")
                    })? as u64);
                },
                _ => {
                    return Err(WebDriverError::new(
                        ErrorStatus::UnknownCommand,
                        "Unknown timeout key",
                    ));
                },
            }
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
    if let Some(script_timeout) = timeouts.script {
        map.insert("script".to_string(), Value::from(script_timeout));
    }
    map.insert("pageLoad".to_string(), Value::from(timeouts.page_load));
    map.insert("implicit".to_string(), Value::from(timeouts.implicit_wait));
    Value::Object(map)
}
