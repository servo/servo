/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::ReferrerPolicy;
use net_traits::request::{CredentialsMode, ParserMetadata};
use script_bindings::str::DOMString;
use servo_url::ServoUrl;

use crate::dom::globalscope::GlobalScope;

#[derive(Clone, Debug, Default, MallocSizeOf)]
pub(crate) enum ScriptFetchPriority {
    High,
    Low,
    #[default]
    Auto,
}

impl From<DOMString> for ScriptFetchPriority {
    fn from(value: DOMString) -> Self {
        match_domstring_ascii!(value,
            "high" => ScriptFetchPriority::High,
            "low" => ScriptFetchPriority::Low,
            "auto" => ScriptFetchPriority::Auto,
            _ => unreachable!(),
        )
    }
}

impl From<&str> for ScriptFetchPriority {
    fn from(value: &str) -> Self {
        match value {
            "high" => ScriptFetchPriority::High,
            "low" => ScriptFetchPriority::Low,
            "auto" => ScriptFetchPriority::Auto,
            _ => Default::default(),
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
/// <https://html.spec.whatwg.org/multipage/#script-fetch-options>
pub(crate) struct ScriptFetchOptions {
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-integrity>
    pub(crate) integrity_metadata: String,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-credentials>
    pub(crate) credentials_mode: CredentialsMode,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-nonce>
    pub(crate) cryptographic_nonce: String,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-parser>
    pub(crate) parser_metadata: ParserMetadata,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-referrer-policy>
    pub(crate) referrer_policy: ReferrerPolicy,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-render-blocking>
    pub(crate) render_blocking: bool,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-fetch-priority>
    pub(crate) fetch_priority: ScriptFetchPriority,
}

impl ScriptFetchOptions {
    /// <https://html.spec.whatwg.org/multipage/#default-script-fetch-options>
    pub(crate) fn default_classic_script() -> ScriptFetchOptions {
        // > The default script fetch options are a script fetch options whose
        // > cryptographic nonce is the empty string,
        // > integrity metadata is the empty string,
        // > parser metadata is "not-parser-inserted",
        // > credentials mode is "same-origin",
        // > referrer policy is the empty string,
        // > and fetch priority is "auto".
        Self {
            cryptographic_nonce: String::new(),
            integrity_metadata: String::new(),
            parser_metadata: ParserMetadata::NotParserInserted,
            credentials_mode: CredentialsMode::CredentialsSameOrigin,
            referrer_policy: ReferrerPolicy::EmptyString,
            render_blocking: false,
            fetch_priority: Default::default(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#get-the-descendant-script-fetch-options>
    pub(crate) fn descendant_fetch_options(
        &self,
        url: &ServoUrl,
        global: &GlobalScope,
    ) -> ScriptFetchOptions {
        // Step 2. Let integrity be the result of resolving a module integrity metadata with url and settingsObject.
        let integrity = global.import_map().resolve_a_module_integrity_metadata(url);

        // Step 1. Let newOptions be a copy of originalOptions.
        // Step 5. Return newOptions.
        Self {
            // Step 3. Set newOptions's integrity metadata to integrity.
            integrity_metadata: integrity,
            cryptographic_nonce: self.cryptographic_nonce.clone(),
            credentials_mode: self.credentials_mode,
            parser_metadata: self.parser_metadata,
            referrer_policy: self.referrer_policy,
            render_blocking: self.render_blocking,
            // Step 4. Set newOptions's fetch priority to "auto".
            fetch_priority: Default::default(),
        }
    }
}

// A private value used by classic and javascript module scripts, to fetch dependencies, resolve
// dynamic imports and module specifiers.
pub(crate) struct BaseScript {
    pub(crate) base_url: ServoUrl,
    pub(crate) options: ScriptFetchOptions,
}

impl BaseScript {
    pub(crate) fn new(base_url: ServoUrl, options: ScriptFetchOptions) -> Self {
        BaseScript { base_url, options }
    }
}
