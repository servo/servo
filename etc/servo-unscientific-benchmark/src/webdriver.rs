/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::time::Duration;

use thirtyfour::{Capabilities, WebDriver};

/// We just have basic webdriver setup here you can reuse in your tests.
pub async fn connect_webdriver_session() -> anyhow::Result<WebDriver> {
    let caps = Capabilities::new();
    let driver = WebDriver::new("http://127.0.0.1:7000", caps).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(driver)
}

/// Simple function to make the current thread sleep
pub async fn sleep(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await
}
