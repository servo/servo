/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, RegExpFlag_IgnoreCase, RegExpFlag_UnicodeSets, RegExpFlags};
use js::rust::HandleObject;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::USVString;
use url::Url;

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
        options: &URLPatternOptions,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<URLPattern>> {
        // Step 1. Set this’s associated URL pattern to the result of create given input, baseURL, and options.
        let pattern = URLPattern::new_with_proto(global, proto, can_gc);
        URLPatternInternal::create(
            input,
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
        URLPattern::initialize(global, proto, input, options, can_gc)
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
    fn create(input: &URLPatternInit, options: &URLPatternOptions, out: &mut Self) -> Fallible<()> {
        // Step 1. Let init be null.
        // Step 2. If input is a scalar value string then:
        // NOTE: We don't support strings as input yet
        // Step 3. Otherwise:
        // Step 3.1 Assert: input is a URLPatternInit.
        // Step 3.2 If baseURL is not null, then throw a TypeError.
        if input.baseURL.is_some() {
            return Err(Error::Type("baseURL must be none".into()));
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
        if default_port == given_port {
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

        // TODO Step 7. Let pattern string be the result of running generate a pattern string given
        // part list and options.
        let pattern_string = Default::default();

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

/// <https://urlpattern.spec.whatwg.org/#parse-a-pattern-string>
fn parse_a_pattern_string(
    input: &str,
    options: Options,
    encoding_callback: EncodingCallback,
) -> Fallible<Vec<Part>> {
    // Step 1. Let parser be a new pattern parser whose encoding callback is encoding callback and
    // segment wildcard regexp is the result of running generate a segment wildcard regexp given options.
    let mut parser = PatternParser::new(
        generate_a_segment_wildcard_regexp(options),
        encoding_callback,
    );

    // Step 2. Set parser’s token list to the result of running tokenize given input and "strict".
    parser.token_list = tokenize(input, TokenizePolicy::Strict)?;

    // TODO: Implement the rest of this algorithm
    Ok(vec![])
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

/// <https://urlpattern.spec.whatwg.org/#process-a-urlpatterninit>
fn process_a_url_pattern_init(
    init: &URLPatternInit,
    init_type: PatternInitType,
) -> Fallible<URLPatternInit> {
    // Step 1. Let result be the result of creating a new URLPatternInit.
    let mut result = URLPatternInit::default();

    // TODO Step 2. If protocol is not null, set result["protocol"] to protocol.
    // TODO Step 3. If username is not null, set result["username"] to username.
    // TODO Step 4. If password is not null, set result["password"] to password.
    // TODO Step 5. If hostname is not null, set result["hostname"] to hostname.
    // TODO Step 6. If port is not null, set result["port"] to port.
    // TODO Step 7. If pathname is not null, set result["pathname"] to pathname.
    // TODO Step 8. If search is not null, set result["search"] to search.
    // TODO Step 9. If hash is not null, set result["hash"] to hash.

    // Step 10. Let baseURL be null.
    let mut base_url: Option<Url> = None;

    // Step 11. If init["baseURL"] exists:
    if let Some(init_base_url) = init.baseURL.as_ref() {
        // Step 11.1 Set baseURL to the result of running the basic URL parser on init["baseURL"].
        let Ok(parsed_base_url) = init_base_url.0.parse() else {
            // Step 11.2 If baseURL is failure, then throw a TypeError.
            return Err(Error::Type(format!(
                "Failed to parse {:?} as URL",
                init_base_url.0
            )));
        };
        let base_url = base_url.insert(parsed_base_url);

        // Step 11.3 If init["protocol"] does not exist, then set result["protocol"] to the result of
        // processing a base URL string given baseURL’s scheme and type.
        if init.protocol.is_none() {
            result.protocol = Some(USVString(process_a_base_url_string(
                base_url.scheme(),
                init_type,
            )));
        }

        // Step 11.4. If type is not "pattern" and init contains none of "protocol", "hostname",
        // "port" and "username",  then set result["username"] to the result of processing a base URL string
        // given baseURL’s username and type.
        if init_type != PatternInitType::Pattern &&
            init.protocol.is_none() &&
            init.hostname.is_none() &&
            init.port.is_none() &&
            init.username.is_none()
        {
            result.username = Some(USVString(process_a_base_url_string(
                base_url.username(),
                init_type,
            )));
        }

        // Step 11.5 If type is not "pattern" and init contains none of "protocol", "hostname", "port",
        // "username" and "password", then set result["password"] to the result of processing a base URL string
        // given baseURL’s password and type.
        if init_type != PatternInitType::Pattern &&
            init.protocol.is_none() &&
            init.hostname.is_none() &&
            init.port.is_none() &&
            init.username.is_none() &&
            init.password.is_none()
        {
            result.password = Some(USVString(process_a_base_url_string(
                base_url.password().unwrap_or_default(),
                init_type,
            )));
        }

        // Step 11.6 If init contains neither "protocol" nor "hostname", then:
        if init.protocol.is_none() && init.hostname.is_none() {
            // Step 11.6.1 Let baseHost be the empty string.
            // Step 11.6.2 If baseURL’s host is not null, then set baseHost to its serialization.
            let base_host = base_url
                .host()
                .map(|host| host.to_string())
                .unwrap_or_default();

            // Step 11.6.3 Set result["hostname"] to the result of processing a base URL string given baseHost and type.
            result.hostname = Some(USVString(process_a_base_url_string(&base_host, init_type)));
        }

        // Step 11.7 If init contains none of "protocol", "hostname", and "port", then:
        if init.protocol.is_none() && init.hostname.is_none() && init.port.is_none() {
            match base_url.port() {
                // Step 11.7.1 If baseURL’s port is null, then set result["port"] to the empty string.
                None => {
                    result.port = Some(USVString(String::new()));
                },
                // Step 11.7.2 Otherwise, set result["port"] to baseURL’s port, serialized.
                Some(port) => {
                    result.port = Some(USVString(port.to_string()));
                },
            }
        }

        // Step 11.8 If init contains none of "protocol", "hostname", "port", and "pathname", then set
        // result["pathname"] to the result of processing a base URL string given the result of
        // URL path serializing baseURL and type.
        if init.protocol.is_none() &&
            init.hostname.is_none() &&
            init.port.is_none() &&
            init.pathname.is_none()
        {
            result.pathname = Some(USVString(process_a_base_url_string(
                base_url.path(),
                init_type,
            )));
        }

        // Step 11.9 If init contains none of "protocol", "hostname", "port", "pathname",
        // and "search", then:
        if init.protocol.is_none() &&
            init.hostname.is_none() &&
            init.port.is_none() &&
            init.pathname.is_none() &&
            init.search.is_none()
        {
            // Step 11.9.1 Let baseQuery be baseURL’s query.
            let base_query = base_url.query();

            // Step 11.9.2 If baseQuery is null, then set baseQuery to the empty string.
            let base_query = base_query.unwrap_or_default();

            // Step 11.9.3 Set result["search"] to the result of processing a base URL string given baseQuery and type.
            result.search = Some(USVString(process_a_base_url_string(base_query, init_type)));
        }

        // Step 11.10 If init contains none of "protocol", "hostname",
        // "port", "pathname", "search", and "hash", then:
        if init.protocol.is_none() &&
            init.hostname.is_none() &&
            init.port.is_none() &&
            init.pathname.is_none() &&
            init.search.is_none() &&
            init.hash.is_none()
        {
            // Step 11.10.1 Let baseFragment be baseURL’s fragment.
            let base_fragment = base_url.fragment();

            // Step 11.10.2 If baseFragment is null, then set baseFragment to the empty string.
            let base_fragment = base_fragment.unwrap_or_default();

            // Step 11.10.3 Set result["hash"] to the result of processing a base URL string
            // given baseFragment and type.
            result.hash = Some(USVString(process_a_base_url_string(
                base_fragment,
                init_type,
            )));
        }
    }

    // Step 12. If init["protocol"] exists, then set result["protocol"] to the result of process protocol for init
    // given init["protocol"] and type.
    if let Some(protocol) = &init.protocol {
        result.protocol = Some(USVString(process_a_protocol_for_init(protocol, init_type)?));
    }

    // Step 13. If init["username"] exists, then set result["username"] to the result of
    // process username for init given init["username"] and type.
    if let Some(username) = &init.username {
        result.username = Some(USVString(process_username_for_init(username, init_type)));
    }

    // Step 14. If init["password"] exists, then set result["password"] to the result of
    // process password for init given init["password"] and type.
    if let Some(password) = &init.password {
        result.password = Some(USVString(process_password_for_init(password, init_type)));
    }

    // Step 15. If init["hostname"] exists, then set result["hostname"] to the result of
    // process hostname for init given init["hostname"] and type.
    if let Some(hostname) = &init.hostname {
        result.hostname = Some(USVString(process_hostname_for_init(hostname, init_type)?));
    }

    // Step 16. Let resultProtocolString be result["protocol"] if it exists; otherwise the empty string.
    let result_protocol_string = result.protocol.as_deref().unwrap_or_default();

    // Step 17. If init["port"] exists, then set result["port"] to the result of process port for init
    // given init["port"], resultProtocolString, and type.
    if let Some(port) = &init.port {
        result.port = Some(USVString(process_port_for_init(
            port,
            result_protocol_string,
            init_type,
        )?));
    }

    // Step 18. If init["pathname"] exists:
    if let Some(path_name) = &init.pathname {
        // Step 18.1 Set result["pathname"] to init["pathname"].
        // NOTE: This is not necessary - the spec uses result["pathname"] in the following section,
        // but it could just as well use init["pathname"]. Storing the string in an intermediate
        // variable makes the code simpler
        let mut result_pathname = path_name.to_string();

        // Step 18.2 If the following are all true:
        // * baseURL is not null;
        // * baseURL does not have an opaque path; and
        // * the result of running is an absolute pathname given result["pathname"] and type is false,
        if let Some(base_url) = base_url {
            if !base_url.cannot_be_a_base() && !is_an_absolute_pathname(path_name, init_type) {
                // Step 18.2.1 Let baseURLPath be the result of running process a base URL string given the result
                // of URL path serializing baseURL and type.
                let base_url_path = process_a_base_url_string(base_url.path(), init_type);

                // Step 18.2.2 Let slash index be the index of the last U+002F (/) code point found in baseURLPath,
                // interpreted as a sequence of code points, or null if there are no instances of the code point.
                let slash_index = base_url_path.rfind('/');

                // Step 18.2.3 If slash index is not null:
                if let Some(slash_index) = slash_index {
                    // Step 18.2.3.1 Let new pathname be the code point substring from 0 to slash index + 1
                    // within baseURLPath.
                    let mut new_pathname = base_url_path[..=slash_index].to_owned();

                    // Step 18.2.3.2 Append result["pathname"] to the end of new pathname.
                    new_pathname.push_str(path_name);

                    // Step 18.2.3.3 Set result["pathname"] to new pathname.
                    result_pathname = new_pathname;
                }
            }
        }

        // Step 18.3 Set result["pathname"] to the result of process pathname for init given result["pathname"],
        // resultProtocolString, and type.
        result.pathname = Some(USVString(process_pathname_for_init(
            &result_pathname,
            result_protocol_string,
            init_type,
        )?));
    }

    // Step 19. If init["search"] exists then set result["search"] to the result of
    // process search for init given init["search"] and type.
    if let Some(search) = &init.search {
        result.search = Some(USVString(process_search_for_init(search, init_type)));
    }

    // Step 20. If init["hash"] exists then set result["hash"] to the result of
    // process hash for init given init["hash"] and type.
    if let Some(hash) = &init.hash {
        result.hash = Some(USVString(process_hash_for_init(hash, init_type)));
    }

    // Step 21. Return result.
    Ok(result)
}

/// <https://urlpattern.spec.whatwg.org/#encoding-callback>
type EncodingCallback = Box<dyn Fn(&str) -> Fallible<String>>;

/// <https://urlpattern.spec.whatwg.org/#token>
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // index isn't used yet, because constructor strings aren't parsed
struct Token<'a> {
    /// <https://urlpattern.spec.whatwg.org/#token-index>
    index: usize,

    /// <https://urlpattern.spec.whatwg.org/#token-value>
    value: &'a str,

    /// <https://urlpattern.spec.whatwg.org/#token-type>
    token_type: TokenType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenType {
    /// <https://urlpattern.spec.whatwg.org/#token-type-open>
    Open,

    /// <https://urlpattern.spec.whatwg.org/#token-type-close>
    Close,

    /// <https://urlpattern.spec.whatwg.org/#token-type-regexp>
    Regexp,

    /// <https://urlpattern.spec.whatwg.org/#token-type-name>
    Name,

    /// <https://urlpattern.spec.whatwg.org/#token-type-char>
    Char,

    /// <https://urlpattern.spec.whatwg.org/#token-type-escaped-char>
    EscapedChar,

    /// <https://urlpattern.spec.whatwg.org/#token-type-other-modifier>
    OtherModifier,

    /// <https://urlpattern.spec.whatwg.org/#token-type-asterisk>
    Asterisk,

    /// <https://urlpattern.spec.whatwg.org/#token-type-end>
    End,

    /// <https://urlpattern.spec.whatwg.org/#token-type-invalid-char>
    InvalidChar,
}

/// <https://urlpattern.spec.whatwg.org/#pattern-parser>
struct PatternParser<'a> {
    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-token-list>
    token_list: Vec<Token<'a>>,
}

/// <https://urlpattern.spec.whatwg.org/#tokenize-policy>
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenizePolicy {
    /// <https://urlpattern.spec.whatwg.org/#tokenize-policy-strict>
    Strict,

    /// <https://urlpattern.spec.whatwg.org/#tokenize-policy-lenient>
    Lenient,
}

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

impl PatternParser<'_> {
    fn new(segment_wildcard_regexp: String, encoding_callback: EncodingCallback) -> Self {
        // This function will look more useful when the parser is implemented
        _ = segment_wildcard_regexp;
        _ = encoding_callback;
        Self { token_list: vec![] }
    }
}

/// <https://urlpattern.spec.whatwg.org/#tokenizer>
struct Tokenizer<'a> {
    input: &'a str,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-policy>
    policy: TokenizePolicy,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-index>
    ///
    /// Note that we deviate the from the spec and index bytes, not code points.
    index: usize,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-next-index>
    ///
    /// Note that we deviate the from the spec and index bytes, not code points.
    next_index: usize,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-token-list>
    token_list: Vec<Token<'a>>,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-code-point>
    code_point: char,
}

/// <https://urlpattern.spec.whatwg.org/#tokenize>
fn tokenize(input: &str, policy: TokenizePolicy) -> Fallible<Vec<Token>> {
    // Step 1. Let tokenizer be a new tokenizer.
    // Step 2. Set tokenizer’s input to input.
    // Step 3. Set tokenizer’s policy to policy.
    let mut tokenizer = Tokenizer {
        input,
        policy,
        index: 0,
        next_index: 0,
        token_list: vec![],
        code_point: char::MIN,
    };

    // Step 4. While tokenizer’s index is less than tokenizer’s input’s code point length:
    while tokenizer.index < tokenizer.input.len() {
        // Step 4.1 Run seek and get the next code point given tokenizer and tokenizer’s index.
        tokenizer.seek_and_get_the_next_code_point(tokenizer.index);

        match tokenizer.code_point {
            // Step 4.2 If tokenizer’s code point is U+002A (*):
            '*' => {
                // Step 4.2.1 Run add a token with default position and length given tokenizer and "asterisk".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Asterisk);

                // Step 4.2.2 Continue.
                continue;
            },
            // Step 4.3 If tokenizer’s code point is U+002B (+) or U+003F (?):
            '+' | '?' => {
                // Step 4.3.1 Run add a token with default position and length given tokenizer and "other-modifier".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::OtherModifier);

                // Step 4.3.2 Continue.
                continue;
            },
            // Step 4.4 If tokenizer’s code point is U+005C (\):
            '\\' => {
                // Step 4.4.1 If tokenizer’s index is equal to tokenizer’s input’s code point length − 1:
                if tokenizer.is_done() {
                    // Step 4.4.1.1 Run process a tokenizing error given tokenizer, tokenizer’s next index,
                    // and tokenizer’s index.
                    tokenizer.process_a_tokenizing_error(tokenizer.next_index, tokenizer.index)?;

                    // Step 4.4.1.2 Continue.
                    continue;
                }

                // Step 4.4.2 Let escaped index be tokenizer’s next index.
                let escaped_index = tokenizer.index;

                // Step 4.4.3 Run get the next code point given tokenizer.
                tokenizer.get_the_next_code_point();

                // Step 4.4.4 Run add a token with default length given tokenizer, "escaped-char",
                // tokenizer’s next index, and escaped index.
                tokenizer.add_a_token_with_default_length(
                    TokenType::EscapedChar,
                    tokenizer.next_index,
                    escaped_index,
                );

                // Step 4.4.5 Continue.
                continue;
            },
            // Step 4.5 If tokenizer’s code point is U+007B ({):
            '{' => {
                // Step 4.5.1 Run add a token with default position and length given tokenizer and "open".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Open);

                // Step 4.5.2 Continue.
                continue;
            },
            // Step 4.6 If tokenizer’s code point is U+007D (}):
            '}' => {
                // Step 4.6.1 Run add a token with default position and length given tokenizer and "close".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Close);

                // Step 4.6.2 Continue.
                continue;
            },
            // Step 4.7 If tokenizer’s code point is U+003A (:):
            ':' => {
                // Step 4.7.1 Let name position be tokenizer’s next index.
                let mut name_position = tokenizer.next_index;

                // Step 4.7.2 Let name start be name position.
                let name_start = name_position;

                // Step 4.7.3 While name position is less than tokenizer’s input’s code point length:
                while name_position < tokenizer.input.len() {
                    // Step 4.7.3.1 Run seek and get the next code point given tokenizer and name position.
                    tokenizer.seek_and_get_the_next_code_point(name_position);

                    // Step 4.7.3.2 Let first code point be true if name position equals name start
                    // and false otherwise.
                    let first_code_point = name_position == name_start;

                    // Step 4.7.3.3 Let valid code point be the result of running is a valid name
                    // code point given tokenizer’s code point and first code point.
                    let valid_code_point =
                        is_a_valid_name_code_point(tokenizer.code_point, first_code_point);

                    // Step 4.7.3.4 If valid code point is false break.
                    if !valid_code_point {
                        break;
                    }

                    // Step 4.6.3.5 Set name position to tokenizer’s next index.
                    name_position = tokenizer.next_index;
                }

                // Step 4.7.4 If name position is less than or equal to name start:
                if name_position <= name_start {
                    // Step 4.7.4.1 Run process a tokenizing error given tokenizer, name start, and tokenizer’s index.
                    tokenizer.process_a_tokenizing_error(name_start, tokenizer.index)?;

                    // Step 4.7.4.2 Continue.
                    continue;
                }

                // Step 4.7.5 Run add a token with default length given tokenizer, "name", name position,
                // and name start.
                tokenizer.add_a_token_with_default_length(
                    TokenType::Name,
                    name_position,
                    name_start,
                );

                // Step 4.7.6 Continue.
                continue;
            },
            // Step 4.8 If tokenizer’s code point is U+0028 (():
            '(' => {
                // Step 4.8.1 Let depth be 1.
                let mut depth = 1;

                // Step 4.8.2 Let regexp position be tokenizer’s next index.
                let mut regexp_position = tokenizer.next_index;

                // Step 4.8.3 Let regexp start be regexp position.
                let regexp_start = regexp_position;

                // Step 4.8.4 Let error be false.
                let mut error = false;

                // Step 4.8.5 While regexp position is less than tokenizer’s input’s code point length:
                while regexp_position < tokenizer.input.len() {
                    // Step 4.8.5.1 Run seek and get the next code point given tokenizer and regexp position.
                    tokenizer.seek_and_get_the_next_code_point(regexp_position);

                    // Step 4.8.5.2 If tokenizer’s code point is not an ASCII code point:
                    if !tokenizer.code_point.is_ascii() {
                        // Step 4.8.5.1.1 Run process a tokenizing error given tokenizer, regexp start,
                        // and tokenizer’s index.
                        tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                        // Step 4.8.5.1.2 Set error to true.
                        error = true;

                        // Step 4.8.5.1.2 Break.
                        break;
                    }

                    // Step 4.8.5.3 If regexp position equals regexp start and tokenizer’s code point is U+003F (?):
                    if regexp_position == regexp_start && tokenizer.code_point == '?' {
                        // Step 4.8.5.3.1 Run process a tokenizing error given tokenizer, regexp start,
                        // and tokenizer’s index.
                        tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                        // Step 4.8.5.3.2 Set error to true.
                        error = true;

                        // Step 4.8.5.3.3 Break.
                        break;
                    }

                    // Step 4.8.5.4 If tokenizer’s code point is U+005C (\):
                    if tokenizer.code_point == '\\' {
                        // Step 4.8.5.4.1 If regexp position equals tokenizer’s input’s code point length − 1:
                        if tokenizer.is_last_character(regexp_position) {
                            // Step 4.8.5.4.1.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.4.1.2 Set error to true.
                            error = true;

                            // Step 4.8.5.4.1.3 Break
                            break;
                        }

                        // Step 4.8.5.4.2 Run get the next code point given tokenizer.
                        tokenizer.get_the_next_code_point();

                        // Step 4.8.5.4.3 If tokenizer’s code point is not an ASCII code point:
                        if !tokenizer.code_point.is_ascii() {
                            // Step 4.8.5.4.3.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.4.3.2 Set error to true.
                            error = true;

                            // Step 4.8.5.4.3.3 Break
                            break;
                        }

                        // Step 4.8.5.4.4 Set regexp position to tokenizer’s next index.
                        regexp_position = tokenizer.next_index;

                        // Step 4.8.5.4.5 Continue.
                        continue;
                    }

                    // Step 4.8.5.5 If tokenizer’s code point is U+0029 ()):
                    if tokenizer.code_point == ')' {
                        // Step 4.8.5.5.1 Decrement depth by 1.
                        depth -= 1;

                        // Step 4.8.5.5.2 If depth is 0:
                        if depth == 0 {
                            // Step 4.8.5.5.2.1 Set regexp position to tokenizer’s next index.
                            regexp_position = tokenizer.next_index;

                            // Step 4.8.5.5.2.2 Break.
                            break;
                        }
                    }
                    // Step 4.8.5.6 Otherwise if tokenizer’s code point is U+0028 (():
                    else if tokenizer.code_point == '(' {
                        // Step 4.8.5.6.1 Increment depth by 1.
                        depth += 1;

                        // Step 4.8.5.6.2 If regexp position equals tokenizer’s input’s code point length − 1:
                        if tokenizer.is_last_character(regexp_position) {
                            // Step 4.8.5.6.2.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.6.2.2 Set error to true.
                            error = true;

                            // Step 4.8.5.6.2.3 Break
                            break;
                        }

                        // Step 4.8.5.6.3 Let temporary position be tokenizer’s next index.
                        let temporary_position = tokenizer.next_index;

                        // Step 4.8.5.6.4 Run get the next code point given tokenizer.
                        tokenizer.get_the_next_code_point();

                        // Step 4.8.5.6.5 If tokenizer’s code point is not U+003F (?):
                        if tokenizer.code_point != '?' {
                            // Step 4.8.5.6.5.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.6.5.2 Set error to true.
                            error = true;

                            // Step 4.8.5.6.5.3 Break.
                            break;
                        }

                        // Step 4.8.5.6.6 Set tokenizer’s next index to temporary position.
                        tokenizer.next_index = temporary_position;
                    }

                    // Step 4.8.5.7 Set regexp position to tokenizer’s next index.
                    regexp_position = tokenizer.next_index;
                }

                // Step 4.8.6 If error is true continue.
                if error {
                    continue;
                }

                // Step 4.8.7 If depth is not zero:
                if depth != 0 {
                    // Step 4.8.7.1 Run process a tokenizing error given tokenizer, regexp start,
                    // and tokenizer’s index
                    tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                    // Step 4.8.7.2 Continue.
                    continue;
                }

                // Step 4.8.8 Let regexp length be regexp position − regexp start − 1.
                let regexp_length = regexp_position - regexp_start - 1;

                // Step 4.8.9 If regexp length is zero:
                if regexp_length == 0 {
                    // Step 4.8.9.1 Run process a tokenizing error given tokenizer, regexp start,
                    // and tokenizer’s index.
                    tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                    // Step 4.8.9.2 Continue.
                    continue;
                }

                // Step 4.8.10 Run add a token given tokenizer, "regexp", regexp position,
                // regexp start, and regexp length.
                tokenizer.add_a_token(
                    TokenType::Regexp,
                    regexp_position,
                    regexp_start,
                    regexp_length,
                );

                // Step 4.8.11 Continue.
                continue;
            },
            _ => {
                // Step 4.9 Run add a token with default position and length given tokenizer and "char".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Char);
            },
        }
    }

    // Step 5. Run add a token with default length given tokenizer, "end", tokenizer’s index, and tokenizer’s index.
    tokenizer.add_a_token_with_default_length(TokenType::End, tokenizer.index, tokenizer.index);

    // Step 6.Return tokenizer’s token list.
    Ok(tokenizer.token_list)
}

/// <https://urlpattern.spec.whatwg.org/#is-a-valid-name-code-point>
fn is_a_valid_name_code_point(code_point: char, first: bool) -> bool {
    // FIXME: implement this check
    _ = first;
    code_point.is_alphabetic()
}

impl Tokenizer<'_> {
    fn is_last_character(&self, position: usize) -> bool {
        self.input[position..].chars().count() == 1
    }

    fn is_done(&self) -> bool {
        self.input[self.next_index..].is_empty()
    }

    /// <https://urlpattern.spec.whatwg.org/#get-the-next-code-point>
    fn get_the_next_code_point(&mut self) {
        // Step 1. Set tokenizer’s code point to the Unicode code point in tokenizer’s
        // input at the position indicated by tokenizer’s next index.
        self.code_point = self.input[self.next_index..]
            .chars()
            .next()
            .expect("URLPattern tokenizer is trying to read out of bounds");

        // Step 2. Increment tokenizer’s next index by 1.
        // NOTE: Because our next_index is indexing bytes (not code points) we use
        // the utf8 length of the code point instead.
        self.next_index = self.next_index.wrapping_add(self.code_point.len_utf8());
    }

    /// <https://urlpattern.spec.whatwg.org/#seek-and-get-the-next-code-point>
    fn seek_and_get_the_next_code_point(&mut self, index: usize) {
        // Step 1. Set tokenizer’s next index to index.
        self.next_index = index;

        // Step 2. Run get the next code point given tokenizer.
        self.get_the_next_code_point();
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-token>
    fn add_a_token(
        &mut self,
        token_type: TokenType,
        next_position: usize,
        value_position: usize,
        value_length: usize,
    ) {
        // Step 1. Let token be a new token.
        // Step 2. Set token’s type to type.
        // Step 3. Set token’s index to tokenizer’s index.
        // Step 4. Set token’s value to the code point substring from value position
        // with length value length within tokenizer’s input.
        let token = Token {
            token_type,
            index: self.index,
            value: &self.input[value_position..][..value_length],
        };

        // Step 5. Append token to the back of tokenizer’s token list.
        self.token_list.push(token);

        // Step 6. Set tokenizer’s index to next position.
        self.index = next_position;
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-token-with-default-position-and-length>
    fn add_a_token_with_default_position_and_length(&mut self, token_type: TokenType) {
        // Step 1. Run add a token with default length given tokenizer, type,
        // tokenizer’s next index, and tokenizer’s index.
        self.add_a_token_with_default_length(token_type, self.next_index, self.index);
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-token-with-default-length>
    fn add_a_token_with_default_length(
        &mut self,
        token_type: TokenType,
        next_position: usize,
        value_position: usize,
    ) {
        // Step 1. Let computed length be next position − value position.
        let computed_length = next_position - value_position;

        // Step 2. Run add a token given tokenizer, type, next position, value position, and computed length.
        self.add_a_token(token_type, next_position, value_position, computed_length);
    }

    /// <https://urlpattern.spec.whatwg.org/#process-a-tokenizing-error>
    fn process_a_tokenizing_error(
        &mut self,
        next_position: usize,
        value_position: usize,
    ) -> Fallible<()> {
        // Step 1. If tokenizer’s policy is "strict", then throw a TypeError.
        if self.policy == TokenizePolicy::Strict {
            return Err(Error::Type("Failed to tokenize URL pattern".into()));
        }

        // Step 2. Assert: tokenizer’s policy is "lenient".
        debug_assert_eq!(self.policy, TokenizePolicy::Lenient);

        // Step 3. Run add a token with default length given tokenizer, "invalid-char",
        // next position, and value position.
        self.add_a_token_with_default_length(TokenType::InvalidChar, next_position, value_position);

        Ok(())
    }
}

/// <https://urlpattern.spec.whatwg.org/#process-a-base-url-string>
fn process_a_base_url_string(input: &str, init_type: PatternInitType) -> String {
    // Step 1. Assert: input is not null.
    // NOTE: The type system ensures that already

    // Step 2. If type is not "pattern" return input.
    if init_type != PatternInitType::Pattern {
        return input.to_owned();
    }

    // Step 3. Return the result of escaping a pattern string given input.
    escape_a_pattern_string(input)
}

/// Implements functionality that is shared between <https://urlpattern.spec.whatwg.org/#escape-a-pattern-string>
/// and <https://urlpattern.spec.whatwg.org/#escape-a-regexp-string>.
///
/// These two algorithms are identical except for the set of characters that they escape, so implementing them
/// seperately does not make sense.
fn escape_a_string(input: &str, to_escape: &[char]) -> String {
    // Step 1. Assert: input is an ASCII string.
    debug_assert!(
        input.is_ascii(),
        "Expected input to be ASCII, got {input:?}"
    );

    // Step 2. Let result be the empty string.
    let mut result = String::with_capacity(input.len());

    // Step 3. Let index be 0.
    // Step 4. While index is less than input’s length:
    // Step 4.1 Let c be input[index].
    // Step 4.2 Increment index by 1.
    for c in input.chars() {
        // Step 4.3 If c is one of: [..] then append "\" to the end of result.
        if to_escape.contains(&c) {
            result.push('\\');
        }

        // Step 4.4 Append c to the end of result.
        result.push(c);
    }

    // Step 5. Return result.
    result
}

/// <https://urlpattern.spec.whatwg.org/#escape-a-pattern-string>
fn escape_a_pattern_string(input: &str) -> String {
    escape_a_string(input, &['+', '*', '?', ':', '{', '}', '(', ')', '\\'])
}

/// <https://urlpattern.spec.whatwg.org/#escape-a-regexp-string>
fn escape_a_regexp_string(input: &str) -> String {
    escape_a_string(
        input,
        &[
            '.', '+', '*', '?', '^', '$', '{', '}', '(', ')', '[', ']', '|', '/', '\\',
        ],
    )
}

/// <https://urlpattern.spec.whatwg.org/#process-protocol-for-init>
fn process_a_protocol_for_init(input: &str, init_type: PatternInitType) -> Fallible<String> {
    // Step 1. Let strippedValue be the given value with a single trailing U+003A (:) removed, if any.
    let stripped_value = input.strip_prefix(':').unwrap_or(input);

    // Step 2. If type is "pattern" then return strippedValue.
    if init_type == PatternInitType::Pattern {
        return Ok(stripped_value.to_owned());
    }

    // Step 3. Return the result of running canonicalize a protocol given strippedValue.
    canonicalize_a_protocol(stripped_value)
}

/// <https://urlpattern.spec.whatwg.org/#process-username-for-init>
fn process_username_for_init(value: &str, init_type: PatternInitType) -> String {
    // Step 1. If type is "pattern" then return value.
    if init_type == PatternInitType::Pattern {
        return value.to_owned();
    }

    // Step 2. Return the result of running canonicalize a username given value.
    canonicalize_a_username(value)
}

/// <https://urlpattern.spec.whatwg.org/#process-password-for-init>
fn process_password_for_init(value: &str, init_type: PatternInitType) -> String {
    // Step 1. If type is "pattern" then return value.
    if init_type == PatternInitType::Pattern {
        return value.to_owned();
    }

    // Step 2. Return the result of running canonicalize a password given value.
    canonicalize_a_password(value)
}

/// <https://urlpattern.spec.whatwg.org/#process-hostname-for-init>
fn process_hostname_for_init(value: &str, init_type: PatternInitType) -> Fallible<String> {
    // Step 1. If type is "pattern" then return value.
    if init_type == PatternInitType::Pattern {
        return Ok(value.to_owned());
    }

    // Step 2. Return the result of running canonicalize a hostname given value.
    canonicalize_a_hostname(value)
}

/// <https://urlpattern.spec.whatwg.org/#process-port-for-init>
fn process_port_for_init(
    port_value: &str,
    protocol_value: &str,
    init_type: PatternInitType,
) -> Fallible<String> {
    // Step 1. If type is "pattern" then return portValue.
    if init_type == PatternInitType::Pattern {
        return Ok(port_value.to_owned());
    }

    // Step 2. Return the result of running canonicalize a port given portValue and protocolValue.
    canonicalize_a_port(port_value, Some(protocol_value))
}

/// <https://urlpattern.spec.whatwg.org/#process-pathname-for-init>
fn process_pathname_for_init(
    path_name_value: &str,
    protocol_value: &str,
    init_type: PatternInitType,
) -> Fallible<String> {
    // Step 1. If type is "pattern" then return pathnameValue.
    if init_type == PatternInitType::Pattern {
        return Ok(path_name_value.to_owned());
    }

    // Step 2. If protocolValue is a special scheme or the empty string, then return the result of
    // running canonicalize a pathname given pathnameValue.
    if is_special_scheme(protocol_value) || protocol_value.is_empty() {
        return Ok(canonicalize_a_pathname(path_name_value));
    }

    // Step 2. Return the result of running canonicalize an opaque pathname given pathnameValue.
    canonicalize_an_opaque_pathname(path_name_value)
}

/// <https://urlpattern.spec.whatwg.org/#process-search-for-init>
fn process_search_for_init(value: &str, init_type: PatternInitType) -> String {
    // Step 1. Let strippedValue be the given value with a single leading U+003F (?) removed, if any.
    let stripped_value = value.strip_prefix('?').unwrap_or(value);

    // Step 2. If type is "pattern" then return strippedValue.
    if init_type == PatternInitType::Pattern {
        return stripped_value.to_owned();
    }

    // Step 3. Return the result of running canonicalize a search given strippedValue.
    canonicalize_a_search(stripped_value)
}

/// <https://urlpattern.spec.whatwg.org/#process-hash-for-init>
fn process_hash_for_init(value: &str, init_type: PatternInitType) -> String {
    // Step 1. Let strippedValue be the given value with a single leading U+0023 (#) removed, if any.
    let stripped_value = value.strip_prefix('#').unwrap_or(value);

    // Step 2. If type is "pattern" then return strippedValue.
    if init_type == PatternInitType::Pattern {
        return stripped_value.to_owned();
    }

    // Step 3. Return the result of running canonicalize a hash given strippedValue.
    canonicalize_a_hash(stripped_value)
}

/// <https://urlpattern.spec.whatwg.org/#url-pattern-create-a-dummy-url>
fn create_a_dummy_url() -> Url {
    // Step 1. Let dummyInput be "https://dummy.invalid/".
    let dummy_input = "https://dummy.invalid/";

    // Step 2. Return the result of running the basic URL parser on dummyInput.
    dummy_input
        .parse()
        .expect("parsing dummy input cannot fail")
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-protocol>
fn canonicalize_a_protocol(value: &str) -> Fallible<String> {
    // Step 1. If value is the empty string, return value.
    if value.is_empty() {
        return Ok(String::new());
    }

    // Step 2. Let parseResult be the result of running the basic URL parser
    // given value followed by "://dummy.invalid/".
    let Ok(parse_result) = Url::parse(&format!("{value}://dummy.invalid/")) else {
        // Step 3. If parseResult is failure, then throw a TypeError.
        return Err(Error::Type(format!(
            "Failed to canonicalize {value:?} as a protocol"
        )));
    };

    // Step 4. Return parseResult’s scheme.
    Ok(parse_result.scheme().to_owned())
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-username>
fn canonicalize_a_username(input: &str) -> String {
    // Step 1. If value is the empty string, return value.
    if input.is_empty() {
        return input.to_owned();
    }

    // Step 2. Let dummyURL be the result of creating a dummy URL.
    let mut dummy_url = create_a_dummy_url();

    // Step 3. Set the username given dummyURL and value.
    dummy_url.set_username(input).unwrap();

    // Step 4. Return dummyURL’s username.
    dummy_url.username().to_owned()
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-password>
fn canonicalize_a_password(input: &str) -> String {
    // Step 1. If value is the empty string, return value.
    if input.is_empty() {
        return input.to_owned();
    }

    // Step 2. Let dummyURL be the result of creating a dummy URL.
    let mut dummy_url = create_a_dummy_url();

    // Step 3. Set the password given dummyURL and value.
    dummy_url.set_password(Some(input)).unwrap();

    // Step 4. Return dummyURL’s password.
    dummy_url.password().unwrap().to_owned()
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-hostname>
fn canonicalize_a_hostname(input: &str) -> Fallible<String> {
    // Step 1. If value is the empty string, return value.
    if input.is_empty() {
        return Ok(String::new());
    }

    // Step 2. Let dummyURL be the result of creating a dummy URL.
    let mut dummy_url = create_a_dummy_url();

    // FIXME: The rest of the algorithm needs functionality that the url crate
    // does not expose. We need to figure out if there's a way around that or
    // if we want to reimplement that functionality here

    if dummy_url.set_host(Some(input)).is_err() {
        return Err(Error::Type(format!(
            "Failed to canonicalize hostname: {input:?}"
        )));
    }

    Ok(dummy_url.host_str().unwrap().to_owned())
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-port>
fn canonicalize_a_port(port_value: &str, protocol_value: Option<&str>) -> Fallible<String> {
    // Step 1. If portValue is the empty string, return portValue.
    if port_value.is_empty() {
        return Ok(String::new());
    }

    // Step 2. Let dummyURL be the result of creating a dummy URL.
    let mut dummy_url = create_a_dummy_url();

    // Step 3. If protocolValue was given, then set dummyURL’s scheme to protocolValue.
    if let Some(protocol_value) = protocol_value {
        dummy_url.set_scheme(protocol_value).unwrap();
    }

    // Step 4. Let parseResult be the result of running basic URL parser given portValue
    // with dummyURL as url and port state as state override.
    // NOTE: The url crate does not expose these parsing concepts, so we try
    // to recreate the parsing step here.
    let port_value = port_value.trim();
    let Ok(port) = port_value.parse::<u16>() else {
        // Step 5. If parseResult is failure, then throw a TypeError.
        return Err(Error::Type(format!(
            "{port_value:?} is not a valid port number"
        )));
    };

    // Step 6. Return dummyURL’s port, serialized, or empty string if it is null.
    if let Some(scheme) = protocol_value {
        if default_port_for_special_scheme(scheme) == Some(port) {
            return Ok(String::new());
        }
    }
    Ok(port.to_string())
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-pathname>
fn canonicalize_a_pathname(value: &str) -> String {
    // Step 1. If value is the empty string, then return value.
    if value.is_empty() {
        return String::new();
    }

    // NOTE: This is not what the spec says, but the url crate does not expose the required functionality.
    // TODO: Investigate whether this is different in practice
    let mut dummy_url = create_a_dummy_url();
    dummy_url.set_path(value);

    dummy_url.path().to_owned()
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-an-opaque-pathname>
fn canonicalize_an_opaque_pathname(value: &str) -> Fallible<String> {
    // NOTE: The url crate doesn't expose the functionality needed by this algorithm.
    // Instead we create a url with an opaque path that is value and then return that opaque path,
    // which should be equivalent.
    let Ok(url) = Url::parse(&format!("foo:{value}")) else {
        return Err(Error::Type(format!(
            "Could not parse {value:?} as opaque path"
        )));
    };

    Ok(url.path().to_owned())
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-search>
fn canonicalize_a_search(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let Ok(url) = Url::parse(&format!("http://example.com?{value}")) else {
        log::warn!("canonicalizing a search should never fail");
        return String::new();
    };

    url.query().unwrap_or_default().to_owned()
}

/// <https://urlpattern.spec.whatwg.org/#canonicalize-a-hash>
fn canonicalize_a_hash(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let Ok(url) = Url::parse(&format!("http://example.com#{value}")) else {
        log::warn!("canonicalizing a hash should never fail");
        return String::new();
    };

    url.fragment().unwrap_or_default().to_owned()
}

/// <https://urlpattern.spec.whatwg.org/#is-an-absolute-pathname>
fn is_an_absolute_pathname(input: &str, init_type: PatternInitType) -> bool {
    let mut chars = input.chars();

    // Step 1. If input is the empty string, then return false.
    let Some(first_char) = chars.next() else {
        return false;
    };

    // Step 2. If input[0] is U+002F (/), then return true.
    if first_char == '/' {
        return true;
    }

    // Step 3. If type is "url", then return false.
    if init_type == PatternInitType::Url {
        return false;
    }

    // Step 4. If input’s code point length is less than 2, then return false.
    let Some(second_char) = chars.next() else {
        return false;
    };

    // Step 5. If input[0] is U+005C (\) and input[1] is U+002F (/), then return true.
    if first_char == '\\' && second_char == '/' {
        return true;
    }

    // Step 6. If input[0] is U+007B ({) and input[1] is U+002F (/), then return true.
    if first_char == '{' && second_char == '/' {
        return true;
    }

    // Step 7. Return false.
    false
}
