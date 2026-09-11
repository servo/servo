/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use anyhow::{Context, Result};
use thirtyfour::{By, WebDriver};

use crate::webdriver::sleep;

pub async fn test(webdriver: &WebDriver) -> Result<()> {
    webdriver.goto("www.pinduoduo.com").await?;
    sleep(5).await;

    let top_bar = webdriver
        .find_all(By::ClassName("menu-item"))
        .await
        .context("Menu Item")?;

    let snd = top_bar.get(1).unwrap();
    snd.click().await.context("Second menu item")?;
    sleep(5).await;

    let windows = webdriver.windows().await?;
    webdriver
        .switch_to_window(windows.get(1).unwrap().clone())
        .await?;

    let footer = webdriver
        .find(By::ClassName("footer-ul"))
        .await
        .context("Footer UI")?;
    footer.scroll_into_view().await?;

    Ok(())
}
