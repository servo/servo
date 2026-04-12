/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::blob_url_store::{BlobResolver, UrlWithBlobClaim};
use servo_url::ServoUrl;

use crate::dom::globalscope::GlobalScope;

pub(crate) fn ensure_blob_referenced_by_url_is_kept_alive(
    global: &GlobalScope,
    url: ServoUrl,
) -> UrlWithBlobClaim {
    match UrlWithBlobClaim::for_url(url) {
        Ok(lock) => lock,
        Err(url) => {
            let token = BlobResolver {
                origin: global.origin().immutable().clone(),
                resource_threads: global.resource_threads(),
            }
            .acquire_blob_token_for(&url);

            UrlWithBlobClaim::new(url, token)
        },
    }
}
