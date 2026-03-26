use net_traits::blob_url_store::{BlobResolver, ServoUrlWithBlobLock};
use servo_url::ServoUrl;

use crate::dom::globalscope::GlobalScope;

pub(crate) fn ensure_blob_referenced_by_url_is_kept_alive(
    global: &GlobalScope,
    url: ServoUrl,
) -> ServoUrlWithBlobLock {
    match ServoUrlWithBlobLock::for_url(url) {
        Ok(lock) => lock,
        Err(url) => {
            let token = BlobResolver {
                origin: global.api_base_url().origin(),
                resource_threads: global.resource_threads(),
            }
            .acquire_blob_token_for(&url);

            ServoUrlWithBlobLock::new(url, token)
        },
    }
}
