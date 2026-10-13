/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::ReferrerPolicy;
use net_traits::request::{CredentialsMode, ParserMetadata};
use script_bindings::refcounted::Trusted;
use servo_url::ServoUrl;

use crate::dom::globalscope::GlobalScope;

#[derive(Clone, Debug, MallocSizeOf)]
/// <https://html.spec.whatwg.org/multipage/#script-fetch-options>
pub(crate) struct ScriptFetchOptions {
    pub(crate) integrity_metadata: String,
    pub(crate) credentials_mode: CredentialsMode,
    pub(crate) cryptographic_nonce: String,
    pub(crate) parser_metadata: ParserMetadata,
    pub(crate) referrer_policy: ReferrerPolicy,
    pub(crate) render_blocking: bool,
}

impl ScriptFetchOptions {
    /// <https://html.spec.whatwg.org/multipage/#default-classic-script-fetch-options>
    pub(crate) fn default_classic_script() -> ScriptFetchOptions {
        Self {
            cryptographic_nonce: String::new(),
            integrity_metadata: String::new(),
            parser_metadata: ParserMetadata::NotParserInserted,
            credentials_mode: CredentialsMode::CredentialsSameOrigin,
            referrer_policy: ReferrerPolicy::EmptyString,
            render_blocking: false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#descendant-script-fetch-options>
    pub(crate) fn descendant_fetch_options(
        &self,
        url: &ServoUrl,
        global: &GlobalScope,
    ) -> ScriptFetchOptions {
        // Step 2. Let integrity be the result of resolving a module integrity metadata with url and settingsObject.
        let integrity = global.import_map().resolve_a_module_integrity_metadata(url);

        // Step 1. Let newOptions be a copy of originalOptions.
        // TODO Step 4. Set newOptions's fetch priority to "auto".
        Self {
            // Step 3. Set newOptions's integrity metadata to integrity.
            integrity_metadata: integrity,
            cryptographic_nonce: self.cryptographic_nonce.clone(),
            credentials_mode: self.credentials_mode,
            parser_metadata: self.parser_metadata,
            referrer_policy: self.referrer_policy,
            render_blocking: self.render_blocking,
        }
    }
}

// A private value used by classic and javascript module scripts, to fetch dependencies, resolve
// dynamic imports and module specifiers.
pub(crate) struct BaseScript {
    pub(crate) base_url: ServoUrl,
    pub(crate) options: ScriptFetchOptions,
    pub(crate) owner: Option<Trusted<GlobalScope>>,
}

impl BaseScript {
    pub(crate) fn new(
        base_url: ServoUrl,
        options: ScriptFetchOptions,
        owner: Option<Trusted<GlobalScope>>,
    ) -> Self {
        BaseScript {
            base_url,
            options,
            owner,
        }
    }
}
