/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use content_security_policy::CspList;
use servo_url::ServoUrl;

use crate::response::Response;
use crate::ReferrerPolicy;

/// <https://html.spec.whatwg.org/multipage/browsers.html#policy-containers>
#[derive(Clone, Debug, Default, MallocSizeOf)]
pub struct PolicyContainer {
    #[ignore_malloc_size_of = "Defined in rust-content-security-policy"]
    csp_list: Option<CspList>,
    referrer_policy: Option<ReferrerPolicy>,
}

impl PolicyContainer {
    pub fn new(csp_list: Option<CspList>, referrer_policy: Option<ReferrerPolicy>) -> Self {
        PolicyContainer {
            csp_list,
            referrer_policy,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/browsers.html#requires-storing-the-policy-container-in-history>
    pub fn requires_storing_in_history(url: &ServoUrl) -> bool {
        if url.scheme() == "blob" {
            false
        } else {
            url.is_local_scheme()
        }
    }

    /// https://html.spec.whatwg.org/multipage/browsers.html#determining-navigation-params-policy-container
    pub fn determine_navigation_params(
        response_url: &ServoUrl,
        history_policy_container: Option<&PolicyContainer>,
        initiator_policy_container: Option<&PolicyContainer>,
        parent_policy_container: Option<&PolicyContainer>,
        response_policy_container: Option<&PolicyContainer>,
    ) -> PolicyContainer {
        // Step 1: If historyPolicyContainer is not null, then:
        if let Some(history_container) = history_policy_container {
            // 1.1: Assert: responseURL requires storing the policy container in history.
            assert!(Self::requires_storing_in_history(response_url));

            // 1.2: Return a clone of historyPolicyContainer.
            return history_container.clone();
        }

        // Step 2: if responseURL is about:srcdoc, then:
        if response_url.as_str() == "about:srcdoc" {
            // 2.1: Assert: parentPolicyContainer is not null.
            assert!(parent_policy_container.is_some());

            // 2.2: Return a clone of parentPolicyContainer.
            return parent_policy_container.unwrap().clone();
        }

        // Step 3: If responseURL is local and initiatorPolicyContainer is not null, then return a
        // clone of initiatorPolicyContainer.
        if response_url.is_local_scheme() {
            if let Some(policy_container) = initiator_policy_container {
                return policy_container.clone();
            }
        }

        // Step 4: If responsePolicyContainer is not null, then return responsePolicyContainer.
        if let Some(policy_container) = response_policy_container {
            return policy_container.clone();
        }

        // Step 5: Return a new policy container.
        return PolicyContainer::new(None, None);
    }
}

/// <https://html.spec.whatwg.org/multipage/browsers.html#creating-a-policy-container-from-a-fetch-response>
impl From<&Response> for PolicyContainer {
    fn from(value: &Response) -> Self {
        // Step 1: If response's URL's scheme is "blob", then return a clone of response's URL's
        // blob URL entry's environment's policy container.

        // Step 2. Let result be a new policy container.

        // Step 3: Set result's CSP list to the result of parsing a response's Content Security
        // Policies given response.

        // Step 4. If environment is non-null, then set result's embedder policy to the result of
        // obtaining an embedder policy given response and environment. Otherwise, set it to
        // "unsafe-none".

        // TODO once cross-origin-embedder-policy is implemented

        // Step 5. Set result's referrer policy to the result of parsing the `Referrer-Policy`
        // header given response.

        // This has been parsed previously and is a property of the request

        // Step 6: Return Result
        return PolicyContainer::new(None, value.referrer_policy);
    }
}
