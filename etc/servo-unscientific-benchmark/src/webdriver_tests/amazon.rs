/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use anyhow::{Context, Result};
use log::info;
use thirtyfour::{By, Key, WebDriver};

use crate::webdriver::sleep;

pub async fn test(webdriver: &WebDriver) -> Result<()> {
    webdriver.goto("https://www.amazon.com").await?;
    sleep(2).await;
    let searchbox = webdriver.find(By::Id("twotabsearchtextbox")).await?;
    searchbox.send_keys("rust").await?;
    searchbox.send_keys(Key::Enter).await?;
    sleep(5).await;
    let pagination = webdriver.find(By::ClassName("s-pagination-item")).await?;
    pagination
        .scroll_into_view()
        .await
        .context("Pagination scroll")?;
    sleep(5).await;
    info!("Finding searchbox");
    let searchbox = webdriver
        .find(By::Id("twotabsearchtextbox"))
        .await
        .context("finding searchbox")?;
    info!("Scrolling up");
    searchbox
        .scroll_into_view()
        .await
        .context("Searchbox scroll")?;
    sleep(5).await;

    // Finding rust book at the top
    webdriver
        .find(By::ClassName("a-link-normal"))
        .await
        .context("Finding link")?;
    sleep(5).await;

    Ok(())
}
