/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod pattern_parser;
mod preprocessing;
mod tokenizer;

use std::ptr;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, RegExpFlag_IgnoreCase, RegExpFlag_UnicodeSets, RegExpFlags};
use js::rust::HandleObject;
use pattern_parser::{generate_a_pattern_string, parse_a_pattern_string};
use preprocessing::{
    canonicalize_a_hash, canonicalize_a_hostname, canonicalize_a_password, canonicalize_a_pathname,
    canonicalize_a_port, canonicalize_a_protocol, canonicalize_a_search, canonicalize_a_username,
    escape_a_regexp_string, process_a_url_pattern_init,
};
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::USVString;

use crate::dom::bindings::cell::RefCell;
use crate::dom::bindings::codegen::Bindings::URLPatternBinding::{
    URLPatternInit, URLPatternMethods, URLPatternOptions,
};
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlinputelement::new_js_regex;

/// <https://urlpattern.spec.whatwg.org/#full-wildcard-regexp-value>
const FULL_WILDCARD_REGEXP_VALUE: &str = ".*";

/// <https://urlpattern.spec.whatwg.org/#urlpattern>
#[dom_struct]
pub(crate) struct URLPattern {
    reflector: Reflector,

    /// <https://urlpattern.spec.whatwg.org/#urlpattern-associated-url-pattern>
    associated_url_pattern: RefCell<URLPatternInternal>,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct URLPatternInternal {
    /// <https://urlpattern.spec.whatwg.org/#url-pattern-protocol-component>
    protocol: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-username-component>
    username: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-password-component>
    password: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-hostname-component>
    hostname: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-port-component>
    port: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-pathname-component>
    pathname: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-search-component>
    search: Component,

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-hash-component>
    hash: Component,
}

/// <https://urlpattern.spec.whatwg.org/#component>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct Component {
    /// <https://urlpattern.spec.whatwg.org/#component-pattern-string>
    pattern_string: USVString,

    /// <https://urlpattern.spec.whatwg.org/#component-regular-expression>
    #[ignore_malloc_size_of = "mozjs"]
    regular_expression: Box<Heap<*mut JSObject>>,

    /// <https://urlpattern.spec.whatwg.org/#component-group-name-list>
    group_name_list: Vec<USVString>,

    /// <https://urlpattern.spec.whatwg.org/#component-has-regexp-groups>
    has_regexp_groups: bool,
}

/// <https://urlpattern.spec.whatwg.org/#part>
#[derive(Debug)]
struct Part {
    /// <https://urlpattern.spec.whatwg.org/#part-type>
    part_type: PartType,

    /// <https://urlpattern.spec.whatwg.org/#part-value>
    value: String,

    /// <https://urlpattern.spec.whatwg.org/#part-modifier>
    modifier: PartModifier,

    /// <https://urlpattern.spec.whatwg.org/#part-name>
    name: String,

    /// <https://urlpattern.spec.whatwg.org/#part-prefix>
    prefix: String,

    /// <https://urlpattern.spec.whatwg.org/#part-suffix>
    suffix: String,
}

/// <https://urlpattern.spec.whatwg.org/#part-type>
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PartType {
    /// <https://urlpattern.spec.whatwg.org/#part-type-fixed-text>
    FixedText,

    /// <https://urlpattern.spec.whatwg.org/#part-type-regexp>
    Regexp,

    /// <https://urlpattern.spec.whatwg.org/#part-type-segment-wildcard>
    SegmentWildcard,

    /// <https://urlpattern.spec.whatwg.org/#part-type-full-wildcard>
    FullWildcard,
}

/// <https://urlpattern.spec.whatwg.org/#part-modifier>
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // Parser is not implemented yet
enum PartModifier {
    /// <https://urlpattern.spec.whatwg.org/#part-modifier-none>
    None,

    /// <https://urlpattern.spec.whatwg.org/#part-modifier-optional>
    Optional,

    /// <https://urlpattern.spec.whatwg.org/#part-modifier-zero-or-more>
    ZeroOrMore,

    /// <https://urlpattern.spec.whatwg.org/#part-modifier-one-or-more>
    OneOrMore,
}

/// <https://urlpattern.spec.whatwg.org/#options>
#[derive(Clone, Copy, Default)]
#[allow(dead_code)] // Parser is not fully implemented yet
struct Options {
    /// <https://urlpattern.spec.whatwg.org/#options-delimiter-code-point>
    delimiter_code_point: Option<char>,

    /// <https://urlpattern.spec.whatwg.org/#options-prefix-code-point>
    prefix_code_point: Option<char>,

    /// <https://urlpattern.spec.whatwg.org/#options-ignore-case>
    ignore_case: bool,
}

impl Component {
    fn new_unrooted() -> Self {
        Self {
            pattern_string: Default::default(),
            regular_expression: Heap::boxed(ptr::null_mut()),
            group_name_list: Default::default(),
            has_regexp_groups: false,
        }
    }
}

impl URLPattern {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited() -> URLPattern {
        let associated_url_pattern = URLPatternInternal {
            protocol: Component::new_unrooted(),
            username: Component::new_unrooted(),
            password: Component::new_unrooted(),
            hostname: Component::new_unrooted(),
            port: Component::new_unrooted(),
            pathname: Component::new_unrooted(),
            search: Component::new_unrooted(),
            hash: Component::new_unrooted(),
        };

        URLPattern {
            reflector: Reflector::new(),
            associated_url_pattern: RefCell::new(associated_url_pattern),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<URLPattern> {
        reflect_dom_object_with_proto(Box::new(URLPattern::new_inherited()), global, proto, can_gc)
    }

    /// <https://urlpattern.spec.whatwg.org/#urlpattern-initialize>
    fn initialize(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        input: &URLPatternInit,
        base_url: Option<USVString>,
        options: &URLPatternOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<URLPattern>> {
        // Step 1. Set this’s associated URL pattern to the result of create given input, baseURL, and options.
        let pattern = URLPattern::new_with_proto(global, proto, can_gc);
        URLPatternInternal::create(
            input,
            base_url,
            options,
            &mut pattern.associated_url_pattern.borrow_mut(),
        )?;

        Ok(pattern)
    }
}

impl URLPatternMethods<crate::DomTypeHolder> for URLPattern {
    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-urlpattern-input-options>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        input: &URLPatternInit,
        options: &URLPatternOptions,
    ) -> Fallible<DomRoot<URLPattern>> {
        // Step 1. Run initialize given this, input, null, and options.
        URLPattern::initialize(global, proto, input, None, options, can_gc)
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-protocol>
    fn Protocol(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s protocol component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .protocol
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-username>
    fn Username(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s username component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .username
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-password>
    fn Password(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s password component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .password
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-hostname>
    fn Hostname(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s hostname component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .hostname
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-port>
    fn Port(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s port component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .port
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-pathname>
    fn Pathname(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s pathname component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .pathname
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-search>
    fn Search(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s search component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .search
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-hash>
    fn Hash(&self) -> USVString {
        // Step 1. Return this’s associated URL pattern’s hash component’s pattern string.
        self.associated_url_pattern
            .borrow()
            .hash
            .pattern_string
            .clone()
    }

    /// <https://urlpattern.spec.whatwg.org/#dom-urlpattern-hasregexpgroups>
    fn HasRegExpGroups(&self) -> bool {
        // Step 1. If this’s associated URL pattern’s has regexp groups, then return true.
        // Step 2. Return false.
        self.associated_url_pattern.borrow().has_regexp_groups()
    }
}

impl URLPatternInternal {
    /// <https://urlpattern.spec.whatwg.org/#url-pattern-create>
    fn create(
        input: &URLPatternInit,
        base_url: Option<USVString>,
        options: &URLPatternOptions,
        out: &mut Self,
    ) -> Fallible<()> {
        // Step 1. Let init be null.
        // Step 2. If input is a scalar value string then:
        // NOTE: We don't support strings as input yet
        // Step 3. Otherwise:
        // Step 3.1 Assert: input is a URLPatternInit.
        // Step 3.2 If baseURL is not null, then throw a TypeError.
        if base_url.is_some() {
            return Err(Error::Type("baseURL must null".into()));
        }

        // Step 3.3 Set init to input.
        let init = input;

        // Step 4. Let processedInit be the result of process a URLPatternInit given init, "pattern", null, null,
        // null, null, null, null, null, and null.
        let mut processed_init = process_a_url_pattern_init(init, PatternInitType::Pattern)?;

        // Step 5. For each componentName of « "protocol", "username", "password", "hostname", "port",
        // "pathname", "search", "hash" »:
        // Step 5.1 If processedInit[componentName] does not exist, then set processedInit[componentName] to "*".
        // NOTE: We do this later on

        // Step 6. If processedInit["protocol"] is a special scheme and processedInit["port"] is a string
        // which represents  its corresponding default port in radix-10 using ASCII digits then set
        // processedInit["port"] to the empty string.
        let default_port = processed_init
            .protocol
            .as_deref()
            .and_then(default_port_for_special_scheme);
        let given_port = processed_init
            .port
            .as_deref()
            .map(str::parse)
            .transpose()
            .ok()
            .flatten();
        if default_port.is_some() && default_port == given_port {
            processed_init.port = Some(Default::default());
        }

        // Step 7. Let urlPattern be a new URL pattern.
        // NOTE: We construct the pattern provided as the out parameter.

        // Step 8. Set urlPattern’s protocol component to the result of compiling a component given
        // processedInit["protocol"], canonicalize a protocol, and default options.
        Component::compile(
            processed_init.protocol.as_deref().unwrap_or("*"),
            Box::new(canonicalize_a_protocol),
            Options::default(),
            &mut out.protocol,
        )?;

        // Step 9. Set urlPattern’s username component to the result of compiling a component given
        // processedInit["username"], canonicalize a username, and default options.
        Component::compile(
            processed_init.username.as_deref().unwrap_or("*"),
            Box::new(|i| Ok(canonicalize_a_username(i))),
            Options::default(),
            &mut out.username,
        )?;

        // Step 10. Set urlPattern’s password component to the result of compiling a component given
        // processedInit["password"], canonicalize a password, and default options.
        Component::compile(
            processed_init.password.as_deref().unwrap_or("*"),
            Box::new(|i| Ok(canonicalize_a_password(i))),
            Options::default(),
            &mut out.password,
        )?;

        // FIXME: Steps 11 and 12: Compile host pattern correctly
        Component::compile(
            processed_init.hostname.as_deref().unwrap_or("*"),
            Box::new(canonicalize_a_hostname),
            Options::HOSTNAME,
            &mut out.hostname,
        )?;

        // Step 13. Set urlPattern’s port component to the result of compiling a component given
        // processedInit["port"], canonicalize a port, and default options.
        Component::compile(
            processed_init.port.as_deref().unwrap_or("*"),
            Box::new(|i| canonicalize_a_port(i, None)),
            Options::default(),
            &mut out.port,
        )?;

        // FIXME: Step 14: respect ignore case option from here on out
        let _ = options;

        // FIXME: Steps 15-16: Compile path pattern correctly
        Component::compile(
            processed_init.pathname.as_deref().unwrap_or("*"),
            Box::new(|i| Ok(canonicalize_a_pathname(i))),
            Options::PATHNAME,
            &mut out.pathname,
        )?;

        // Step 17. Set urlPattern’s search component to the result of compiling a component given
        // processedInit["search"], canonicalize a search, and compileOptions.
        Component::compile(
            processed_init.search.as_deref().unwrap_or("*"),
            Box::new(|i| Ok(canonicalize_a_search(i))),
            Options::default(),
            &mut out.search,
        )?;

        // Step 18. Set urlPattern’s hash component to the result of compiling a component given
        // processedInit["hash"], canonicalize a hash, and compileOptions.
        Component::compile(
            processed_init.hash.as_deref().unwrap_or("*"),
            Box::new(|i| Ok(canonicalize_a_hash(i))),
            Options::default(),
            &mut out.hash,
        )?;

        // Step 19. Return urlPattern.
        // NOTE: not necessary since we use an out parameter
        Ok(())
    }

    /// <https://urlpattern.spec.whatwg.org/#url-pattern-has-regexp-groups>
    fn has_regexp_groups(&self) -> bool {
        self.protocol.has_regexp_groups ||
            self.username.has_regexp_groups ||
            self.password.has_regexp_groups ||
            self.hostname.has_regexp_groups ||
            self.port.has_regexp_groups ||
            self.pathname.has_regexp_groups ||
            self.search.has_regexp_groups ||
            self.hash.has_regexp_groups
    }
}

impl Component {
    /// <https://urlpattern.spec.whatwg.org/#compile-a-component>
    fn compile(
        input: &str,
        encoding_callback: EncodingCallback,
        options: Options,
        out: &mut Self,
    ) -> Fallible<()> {
        // Step 1. Let part list be the result of running parse a pattern string given input, options,
        // and encoding callback.
        let part_list = parse_a_pattern_string(input, options, encoding_callback)?;

        // Step 2. Let (regular expression string, name list) be the result of running generate a regular expression and
        // name list given part list and options.
        let (regular_expression_string, name_list) =
            generate_a_regular_expression_and_name_list(&part_list, options);

        log::debug!("Compiled {input:?} (URLPattern) to {regular_expression_string:?} (Regex)");

        // Step 3. Let flags be an empty string.
        // Step 4. If options’s ignore case is true then set flags to "vi".
        let flags = if options.ignore_case {
            RegExpFlags {
                flags_: RegExpFlag_UnicodeSets | RegExpFlag_IgnoreCase,
            }
        }
        // Step 5. Otherwise set flags to "v"
        else {
            RegExpFlags {
                flags_: RegExpFlag_UnicodeSets,
            }
        };

        // Step 6. Let regular expression be RegExpCreate(regular expression string, flags).
        // If this throws an exception, catch it, and throw a TypeError.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut regular_expression: *mut JSObject = ptr::null_mut());
        let succeeded = new_js_regex(
            cx,
            &regular_expression_string,
            flags,
            regular_expression.handle_mut(),
        );
        if !succeeded {
            return Err(Error::Type(format!(
                "Failed to compile {regular_expression_string:?} as a regular expression"
            )));
        }

        // Step 7. Let pattern string be the result of running generate a pattern string given
        // part list and options.
        let pattern_string = USVString(generate_a_pattern_string(&part_list, options));

        // Step 8. Let has regexp groups be false.
        // Step 9. For each part of part list:
        // Step 9.1 If part’s type is "regexp", then set has regexp groups to true.
        let has_regexp_groups = part_list
            .iter()
            .any(|part| part.part_type == PartType::Regexp);

        // Step 10. Return a new component whose pattern string is pattern string, regular expression
        // is regular expression, group name list is name list, and has regexp groups is has regexp groups.
        out.pattern_string = pattern_string;
        out.regular_expression.set(*regular_expression.handle());
        out.group_name_list = name_list;
        out.has_regexp_groups = has_regexp_groups;

        Ok(())
    }
}

/// <https://urlpattern.spec.whatwg.org/#generate-a-regular-expression-and-name-list>
fn generate_a_regular_expression_and_name_list(
    part_list: &[Part],
    options: Options,
) -> (String, Vec<USVString>) {
    // Step 1. Let result be "^".
    let mut result = String::from("^");

    // Step 2. Let name list be a new list.
    let mut name_list = vec![];

    // Step 3. For each part of part list:
    for part in part_list {
        // Step 3.1 If part’s type is "fixed-text":
        if part.part_type == PartType::FixedText {
            // Step 3.1.1 If part’s modifier is "none", then append the result of running escape a regexp string given
            // part’s value to the end of result.
            if part.modifier == PartModifier::None {
                result.push_str(&escape_a_regexp_string(&part.value));
            }
            // Step 3.1.2 Otherwise:
            else {
                // Step 3.1.2.1 Append "(?:" to the end of result.
                result.push_str("(?:");

                // Step 3.1.2.2 Append the result of running escape a regexp string given part’s value
                // to the end of result.
                result.push_str(&escape_a_regexp_string(&part.value));

                // Step 3.1.2.3 Append ")" to the end of result.
                result.push(')');

                // Step 3.1.2.4 Append the result of running convert a modifier to a string given part’s
                // modifier to the end of result.
                result.push_str(part.modifier.convert_to_string());
            }

            // Step 3.1.3 Continue.
            continue;
        }

        // Step 3.2 Assert: part’s name is not the empty string.
        debug_assert!(!part.name.is_empty());

        // Step 3.3 Append part’s name to name list.
        name_list.push(USVString(part.name.to_string()));

        // Step 3.4 Let regexp value be part’s value.
        let mut regexp_value = part.value.clone();

        // Step 3.5 If part’s type is "segment-wildcard", then set regexp value to the result of running
        // generate a segment wildcard regexp given options.
        if part.part_type == PartType::SegmentWildcard {
            regexp_value = generate_a_segment_wildcard_regexp(options);
        }
        // Step 3.6 Otherwise if part’s type is "full-wildcard", then set regexp value to full wildcard regexp value.
        else if part.part_type == PartType::FullWildcard {
            regexp_value = FULL_WILDCARD_REGEXP_VALUE.into();
        }

        // Step 3.7 If part’s prefix is the empty string and part’s suffix is the empty string:
        if part.prefix.is_empty() && part.suffix.is_empty() {
            // Step 3.7.1 If part’s modifier is "none" or "optional", then:
            if matches!(part.modifier, PartModifier::None | PartModifier::Optional) {
                // Step 3.7.1.1 Append "(" to the end of result.
                result.push('(');

                // Step 3.7.1.2 Append regexp value to the end of result.
                result.push_str(&regexp_value);

                // Step 3.7.1.3 Append ")" to the end of result.
                result.push(')');

                // Step 3.7.1.4 Append the result of running convert a modifier to a string given part’s modifier
                // to the end of result.
                result.push_str(part.modifier.convert_to_string());
            }
            // Step 3.7.2 Otherwise:
            else {
                // Step 3.7.2.1 Append "((?:" to the end of result.
                result.push_str("((?:");

                // Step 3.7.2.2 Append regexp value to the end of result.
                result.push_str(&regexp_value);

                // Step 3.7.2.3 Append ")" to the end of result.
                result.push(')');

                // Step 3.7.2.4 Append the result of running convert a modifier to a string given part’s modifier
                // to the end of result.
                result.push_str(part.modifier.convert_to_string());

                // Step 3.7.2.5 Append ")" to the end of result.
                result.push(')');
            }

            // Step 3.7.3 Continue.
            continue;
        }

        // Step 3.8 If part’s modifier is "none" or "optional":
        if matches!(part.modifier, PartModifier::None | PartModifier::Optional) {
            // Step 3.8.1 Append "(?:" to the end of result.
            result.push_str("(?:");

            // Step 3.8.2 Append the result of running escape a regexp string given part’s prefix
            // to the end of result.
            result.push_str(&escape_a_regexp_string(&part.prefix));

            // Step 3.8.3 Append "(" to the end of result.
            result.push('(');

            // Step 3.8.4 Append regexp value to the end of result.
            result.push_str(&regexp_value);

            // Step 3.8.5 Append ")" to the end of result.
            result.push(')');

            // Step 3.8.6 Append the result of running escape a regexp string given part’s suffix
            // to the end of result.
            result.push_str(&escape_a_regexp_string(&part.suffix));

            // Step 3.8.7 Append ")" to the end of result.
            result.push(')');

            // Step 3.8.8 Append the result of running convert a modifier to a string given part’s modifier to
            // the end of result.
            result.push_str(part.modifier.convert_to_string());

            // Step 3.8.9 Continue.
            continue;
        }

        // Step 3.9 Assert: part’s modifier is "zero-or-more" or "one-or-more".
        debug_assert!(matches!(
            part.modifier,
            PartModifier::ZeroOrMore | PartModifier::OneOrMore
        ));

        // Step 3.10 Assert: part’s prefix is not the empty string or part’s suffix is not the empty string.
        debug_assert!(!part.prefix.is_empty() || !part.suffix.is_empty());

        // Step 3.11 Append "(?:" to the end of result.
        result.push_str("(?:");

        // Step 3.12 Append the result of running escape a regexp string given part’s prefix to the end of result.
        result.push_str(&escape_a_regexp_string(&part.prefix));

        // Step 3.13 Append "((?:" to the end of result.
        result.push_str("((?:");

        // Step 3.14 Append regexp value to the end of result.
        result.push_str(&regexp_value);

        // Step 3.15 Append ")(?:" to the end of result.
        result.push_str(")(?:");

        // Step 3.16 Append the result of running escape a regexp string given part’s suffix to the end of result.
        result.push_str(&escape_a_regexp_string(&part.suffix));

        // Step 3.17 Append the result of running escape a regexp string given part’s prefix to the end of result.
        result.push_str(&escape_a_regexp_string(&part.prefix));

        // Step 3.18 Append "(?:" to the end of result.
        result.push_str("(?:");

        // Step 3.19 Append regexp value to the end of result.
        result.push_str(&regexp_value);

        // Step 3.20 Append "))*)" to the end of result.
        result.push_str("))*)");

        // Step 3.21 Append the result of running escape a regexp string given part’s suffix to the end of result.
        result.push_str(&escape_a_regexp_string(&part.suffix));

        // Step 3.22 Append ")" to the end of result.
        result.push(')');

        // Step 3.23 If part’s modifier is "zero-or-more" then append "?" to the end of result.
        if part.modifier == PartModifier::ZeroOrMore {
            result.push('?');
        }
    }

    // Step 4. Append "$" to the end of result.
    result.push('$');

    // Step 5. Return (result, name list).
    (result, name_list)
}

/// <https://urlpattern.spec.whatwg.org/#encoding-callback>
type EncodingCallback = Box<dyn Fn(&str) -> Fallible<String>>;

// FIXME: Deduplicate this with the url crate
/// <https://url.spec.whatwg.org/#special-scheme>
fn default_port_for_special_scheme(scheme: &str) -> Option<u16> {
    match scheme {
        "ftp" => Some(21),
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        _ => None,
    }
}

/// <https://url.spec.whatwg.org/#special-scheme>
fn is_special_scheme(scheme: &str) -> bool {
    matches!(scheme, "ftp" | "http" | "https" | "ws" | "wss")
}

/// <https://urlpattern.spec.whatwg.org/#generate-a-segment-wildcard-regexp>
fn generate_a_segment_wildcard_regexp(options: Options) -> String {
    // Step 1. Let result be "[^".
    let mut result = String::from("[^");

    // Step 2. Append the result of running escape a regexp string given options’s
    // delimiter code point to the end of result.
    result.push_str(&escape_a_regexp_string(
        &options
            .delimiter_code_point
            .map(|c| c.to_string())
            .unwrap_or_default(),
    ));

    // Step 3. Append "]+?" to the end of result.
    result.push_str("]+?");

    // Step 4. Return result.
    result
}

impl PartModifier {
    /// <https://urlpattern.spec.whatwg.org/#convert-a-modifier-to-a-string>
    fn convert_to_string(&self) -> &'static str {
        match self {
            // Step 1. If modifier is "zero-or-more", then return "*".
            Self::ZeroOrMore => "*",
            // Step 2. If modifier is "optional", then return "?".
            Self::Optional => "?",
            // Step 3. If modifier is "one-or-more", then return "+".
            Self::OneOrMore => "+",
            // Step 4. Return the empty string.
            _ => "",
        }
    }
}

impl Options {
    /// <https://urlpattern.spec.whatwg.org/#hostname-options>
    const HOSTNAME: Self = Self {
        delimiter_code_point: Some('.'),
        prefix_code_point: None,
        ignore_case: false,
    };

    /// <https://urlpattern.spec.whatwg.org/#pathname-options>
    const PATHNAME: Self = Self {
        delimiter_code_point: Some('/'),
        prefix_code_point: Some('/'),
        ignore_case: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PatternInitType {
    Pattern,
    Url,
}

impl Part {
    fn new(part_type: PartType, value: String, modifier: PartModifier) -> Self {
        Self {
            part_type,
            value,
            modifier,
            name: String::new(),
            prefix: String::new(),
            suffix: String::new(),
        }
    }
}
