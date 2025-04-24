/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell};
use std::collections::BTreeSet;

use serde::Serialize;
use servo_url::ServoUrl;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceData {
    pub actor: String,
    /// URL of the script, or URL of the page for inline scripts.
    pub url: String,
    pub is_black_boxed: bool,
}

#[derive(Serialize)]
pub(crate) struct SourcesReply {
    pub from: String,
    pub sources: Vec<SourceData>,
}

pub(crate) struct Source {
    actor_name: String,
    source_urls: RefCell<BTreeSet<SourceData>>,
}

impl Source {
    pub fn new(actor_name: String) -> Self {
        Self {
            actor_name,
            source_urls: RefCell::new(BTreeSet::default()),
        }
    }

    pub fn add_source(&self, url: ServoUrl) {
        self.source_urls.borrow_mut().insert(SourceData {
            actor: self.actor_name.clone(),
            url: url.to_string(),
            is_black_boxed: false,
        });
    }

    pub fn sources(&self) -> Ref<BTreeSet<SourceData>> {
        self.source_urls.borrow()
    }
}
