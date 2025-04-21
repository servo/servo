/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::USVString;

use crate::dom::bindings::codegen::Bindings::URLPatternBinding;
use crate::dom::bindings::codegen::Bindings::URLPatternBinding::URLPatternMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::globalscope::GlobalScope;

/// <https://urlpattern.spec.whatwg.org/#full-wildcard-regexp-value>
const FULL_WILDCARD_REGEXP_VALUE: &str = ".*";

/// <https://urlpattern.spec.whatwg.org/#urlpattern>
#[dom_struct]
pub(crate) struct URLPattern {
    reflector: Reflector,

    /// <https://urlpattern.spec.whatwg.org/#urlpattern-associated-url-pattern>
    #[no_trace]
    #[ignore_malloc_size_of = "defined in urlpattern crate"]
    associated_url_pattern: urlpattern::UrlPattern,
}

impl URLPattern {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        init: urlpattern::UrlPatternInit,
        options: urlpattern::UrlPatternOptions,
    ) -> Fallible<URLPattern> {
        let associated_url_pattern =
            urlpattern::UrlPattern::parse(init, options).map_err(|error| {
                log::warn!("Failed to parse URLPattern: {error}");
                Error::Type(format!("{error}"))
            })?;

        let pattern = URLPattern {
            reflector: Reflector::new(),
            associated_url_pattern,
        };
        Ok(pattern)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: urlpattern::UrlPatternInit,
        options: urlpattern::UrlPatternOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<URLPattern>> {
        let url_pattern = reflect_dom_object_with_proto(
            Box::new(URLPattern::new_inherited(init, options)?),
            global,
            proto,
            can_gc,
        );
        Ok(url_pattern)
    }
}

impl URLPatternMethods<crate::DomTypeHolder> for URLPattern {
    // // <https://urlpattern.spec.whatwg.org/#dom-urlpattern-urlpattern>
    // fn Constructor(
    //     global: &GlobalScope,
    //     proto: Option<HandleObject>,
    //     can_gc: CanGc,
    //     input: &URLPatternBinding::URLPatternInit,
    //     base_url: USVString,
    //     options: &URLPatternBinding::URLPatternOptions,
    // ) -> Fallible<DomRoot<URLPattern>> {
    //     // Step 1. Run initialize given this, input, baseURL, and options.
    //     URLPattern::new_with_proto(global, proto, input, Some(base_url), options, can_gc)
    // }
    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-urlpattern-input-options>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        input: &URLPatternBinding::URLPatternInit,
        options: &URLPatternBinding::URLPatternOptions,
    ) -> Fallible<DomRoot<URLPattern>> {
        // Step 1. Run initialize given this, input, null, and options.
        let base_url = input
            .baseURL
            .as_ref()
            .map(|base_url| base_url.parse().map_err(|e| Error::Type(format!("{e}"))))
            .transpose()?;

        let input = urlpattern::UrlPatternInit {
            protocol: input
                .protocol
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            username: input
                .username
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            password: input
                .password
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            hostname: input
                .hostname
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            port: input.port.as_ref().map(|usv_string| usv_string.to_string()),
            pathname: input
                .pathname
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            search: input
                .search
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            hash: input.hash.as_ref().map(|usv_string| usv_string.to_string()),
            base_url,
        };
        let options = urlpattern::UrlPatternOptions {
            ignore_case: options.ignoreCase,
        };
        URLPattern::new_with_proto(global, proto, input, options, can_gc)
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-protocol>
    fn Protocol(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s protocol component’s pattern string.
        USVString(self.associated_url_pattern.protocol().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-username>
    fn Username(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s username component’s pattern string.
        USVString(self.associated_url_pattern.username().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-password>
    fn Password(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s password component’s pattern string.
        USVString(self.associated_url_pattern.password().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-hostname>
    fn Hostname(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s hostname component’s pattern string.
        USVString(self.associated_url_pattern.hostname().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-port>
    fn Port(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s port component’s pattern string.
        USVString(self.associated_url_pattern.port().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-pathname>
    fn Pathname(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s pathname component’s pattern string.
        USVString(self.associated_url_pattern.pathname().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-search>
    fn Search(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s search component’s pattern string.
        USVString(self.associated_url_pattern.search().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-hash>
    fn Hash(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s hash component’s pattern string.
        USVString(self.associated_url_pattern.hash().to_owned())
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-hasregexpgroups>
    fn HasRegExpGroups(&self) -> bool {
        // Step 1. If this’s associated URL pattern’s has regexp groups, then return true.
        // Step 2. Return false.
        self.associated_url_pattern.has_regexp_groups()
    }
}
