/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use anyhow::Result;
use thirtyfour::{By, Key, WebDriver};

use crate::webdriver::sleep;

pub async fn test(webdriver: &WebDriver) -> Result<()> {
    webdriver.goto("www.ebay.com").await?;
    sleep(5).await;
    let searchbox = webdriver.find(By::Id("gh-ac")).await?;
    searchbox.send_keys("servo").await?;
    searchbox.send_keys(Key::Enter).await?;

    sleep(5).await;
    let pageination_items = webdriver
        .find_all(By::ClassName("pagination__item"))
        .await?;
    pageination_items
        .first()
        .unwrap()
        .scroll_into_view()
        .await?;
    sleep(5).await;
    pageination_items.get(1).unwrap().click().await?;

    let pageination2 = webdriver.find(By::ClassName("pagination__item")).await?;
    pageination2.scroll_into_view().await?;
    Ok(())
}
