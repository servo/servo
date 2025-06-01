/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::URLPatternBinding::URLPatternResult;
use script_bindings::codegen::GenericUnionTypes::USVStringOrURLPatternInit;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::USVString;

use crate::dom::bindings::codegen::Bindings::URLPatternBinding;
use crate::dom::bindings::codegen::Bindings::URLPatternBinding::URLPatternMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::globalscope::GlobalScope;

/// <https://urlpattern.spec.whatwg.org/#urlpattern>
#[dom_struct]
pub(crate) struct URLPattern {
    reflector: Reflector,

    /// <https://urlpattern.spec.whatwg.org/#urlpattern-associated-url-pattern>
    #[no_trace]
    associated_url_pattern: urlpattern::UrlPattern,
}

impl URLPattern {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(associated_url_pattern: urlpattern::UrlPattern) -> URLPattern {
        URLPattern {
            reflector: Reflector::new(),
            associated_url_pattern,
        }
    }

    /// <https://urlpattern.spec.whatwg.org/#urlpattern-initialize>
    pub(crate) fn initialize(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        input: USVStringOrURLPatternInit,
        base_url: Option<USVString>,
        options: &URLPatternBinding::URLPatternOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<URLPattern>> {
        // The section below converts from servos types to the types used in the urlpattern crate
        let base_url = base_url.map(|usv_string| usv_string.0);
        let input = bindings_to_third_party::map_urlpattern_input(input);
        let options = urlpattern::UrlPatternOptions {
            ignore_case: options.ignoreCase,
        };

        // Parse and initialize the URL pattern.
        let pattern_init =
            urlpattern::quirks::process_construct_pattern_input(input, base_url.as_deref())
                .map_err(|error| Error::Type(format!("{error}")))?;

        let pattern = urlpattern::UrlPattern::parse(pattern_init, options)
            .map_err(|error| Error::Type(format!("{error}")))?;

        let url_pattern = reflect_dom_object_with_proto(
            Box::new(URLPattern::new_inherited(pattern)),
            global,
            proto,
            can_gc,
        );
        Ok(url_pattern)
    }
}

impl URLPatternMethods<crate::DomTypeHolder> for URLPattern {
    // <https://urlpattern.spec.whatwg.org/#dom-urlpattern-urlpattern>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        input: USVStringOrURLPatternInit,
        base_url: USVString,
        options: &URLPatternBinding::URLPatternOptions,
    ) -> Fallible<DomRoot<URLPattern>> {
        URLPattern::initialize(global, proto, input, Some(base_url), options, can_gc)
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-urlpattern-input-options>
    fn Constructor_(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        input: USVStringOrURLPatternInit,
        options: &URLPatternBinding::URLPatternOptions,
    ) -> Fallible<DomRoot<URLPattern>> {
        // Step 1. Run initialize given this, input, null, and options.
        URLPattern::initialize(global, proto, input, None, options, can_gc)
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-test>
    fn Test(
        &self,
        input: USVStringOrURLPatternInit,
        base_url: Option<USVString>,
    ) -> Fallible<bool> {
        let input = bindings_to_third_party::map_urlpattern_input(input);
        let inputs = urlpattern::quirks::process_match_input(input, base_url.as_deref())
            .map_err(|error| Error::Type(format!("{error}")))?;
        let Some((match_input, _)) = inputs else {
            return Ok(false);
        };

        self.associated_url_pattern
            .test(match_input)
            .map_err(|error| Error::Type(format!("{error}")))
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-exec>
    fn Exec(
        &self,
        input: USVStringOrURLPatternInit,
        base_url: Option<USVString>,
    ) -> Fallible<Option<URLPatternResult>> {
        let input = bindings_to_third_party::map_urlpattern_input(input);
        let inputs = urlpattern::quirks::process_match_input(input, base_url.as_deref())
            .map_err(|error| Error::Type(format!("{error}")))?;
        let Some((match_input, inputs)) = inputs else {
            return Ok(None);
        };

        let result = self
            .associated_url_pattern
            .exec(match_input)
            .map_err(|error| Error::Type(format!("{error}")))?;
        let Some(result) = result else {
            return Ok(None);
        };

        Ok(Some(third_party_to_bindings::map_urlpattern_result(
            result, inputs,
        )))
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

mod bindings_to_third_party {
    use script_bindings::codegen::GenericBindings::URLPatternBinding::URLPatternInit;

    use crate::dom::urlpattern::USVStringOrURLPatternInit;

    fn map_urlpatterninit(pattern_init: URLPatternInit) -> urlpattern::quirks::UrlPatternInit {
        urlpattern::quirks::UrlPatternInit {
            protocol: pattern_init.protocol.map(|protocol| protocol.0),
            username: pattern_init.username.map(|username| username.0),
            password: pattern_init.password.map(|password| password.0),
            hostname: pattern_init.hostname.map(|hostname| hostname.0),
            port: pattern_init.port.map(|hash| hash.0),
            pathname: pattern_init
                .pathname
                .as_ref()
                .map(|usv_string| usv_string.to_string()),
            search: pattern_init.search.map(|search| search.0),
            hash: pattern_init.hash.map(|hash| hash.0),
            base_url: pattern_init.baseURL.map(|base_url| base_url.0),
        }
    }

    pub(super) fn map_urlpattern_input(
        input: USVStringOrURLPatternInit,
    ) -> urlpattern::quirks::StringOrInit {
        match input {
            USVStringOrURLPatternInit::USVString(usv_string) => {
                urlpattern::quirks::StringOrInit::String(usv_string.0)
            },
            USVStringOrURLPatternInit::URLPatternInit(pattern_init) => {
                urlpattern::quirks::StringOrInit::Init(map_urlpatterninit(pattern_init))
            },
        }
    }
}

mod third_party_to_bindings {
    use script_bindings::codegen::GenericBindings::URLPatternBinding::{
        URLPatternComponentResult, URLPatternInit, URLPatternResult,
    };
    use script_bindings::codegen::GenericUnionTypes::USVStringOrUndefined;
    use script_bindings::record::Record;
    use script_bindings::str::USVString;

    use crate::dom::bindings::codegen::UnionTypes::USVStringOrURLPatternInit;

    // FIXME: For some reason codegen puts a lot of options into these types that don't make sense

    fn map_component_result(
        component_result: urlpattern::UrlPatternComponentResult,
    ) -> URLPatternComponentResult {
        let mut groups = Record::new();
        for (key, value) in component_result.groups.iter() {
            let value = match value {
                Some(value) => USVStringOrUndefined::USVString(USVString(value.to_owned())),
                None => USVStringOrUndefined::Undefined(()),
            };

            groups.insert(USVString(key.to_owned()), value);
        }

        URLPatternComponentResult {
            input: Some(component_result.input.into()),
            groups: Some(groups),
        }
    }

    fn map_urlpatterninit(pattern_init: urlpattern::quirks::UrlPatternInit) -> URLPatternInit {
        URLPatternInit {
            baseURL: pattern_init.base_url.map(USVString),
            protocol: pattern_init.protocol.map(USVString),
            username: pattern_init.username.map(USVString),
            password: pattern_init.password.map(USVString),
            hostname: pattern_init.hostname.map(USVString),
            port: pattern_init.port.map(USVString),
            pathname: pattern_init.pathname.map(USVString),
            search: pattern_init.search.map(USVString),
            hash: pattern_init.hash.map(USVString),
        }
    }

    pub(super) fn map_urlpattern_result(
        result: urlpattern::UrlPatternResult,
        (string_or_init, base_url): urlpattern::quirks::Inputs,
    ) -> URLPatternResult {
        let string_or_init = match string_or_init {
            urlpattern::quirks::StringOrInit::String(string) => {
                USVStringOrURLPatternInit::USVString(USVString(string))
            },
            urlpattern::quirks::StringOrInit::Init(pattern_init) => {
                USVStringOrURLPatternInit::URLPatternInit(map_urlpatterninit(pattern_init))
            },
        };

        let mut inputs = vec![string_or_init];

        if let Some(base_url) = base_url {
            inputs.push(USVStringOrURLPatternInit::USVString(USVString(base_url)));
        }

        URLPatternResult {
            inputs: Some(inputs),
            protocol: Some(map_component_result(result.protocol)),
            username: Some(map_component_result(result.username)),
            password: Some(map_component_result(result.password)),
            hostname: Some(map_component_result(result.hostname)),
            port: Some(map_component_result(result.port)),
            pathname: Some(map_component_result(result.pathname)),
            search: Some(map_component_result(result.search)),
            hash: Some(map_component_result(result.hash)),
        }
    }
}
