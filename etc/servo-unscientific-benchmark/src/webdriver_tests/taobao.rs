/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use anyhow::Result;
use thirtyfour::{By, Key, WebDriver};

use crate::webdriver::sleep;

pub async fn test(webdriver: &WebDriver) -> Result<()> {
    webdriver.goto("www.taobao.com").await?;
    sleep(5).await;
    let searchbox = webdriver.find(By::Id("q")).await?;
    searchbox.send_keys("servo").await?;
    searchbox.send_keys(Key::Enter).await?;

    sleep(5).await;
    Ok(())
}
