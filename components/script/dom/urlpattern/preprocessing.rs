/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::error::{Error, Fallible};
use script_bindings::str::USVString;
use url::Url;

use crate::dom::bindings::codegen::Bindings::URLPatternBinding::URLPatternInit;
use crate::dom::urlpattern::{PatternInitType, default_port_for_special_scheme, is_special_scheme};

/// <https://urlpattern.spec.whatwg.org/#process-a-urlpatterninit>
pub(super) fn process_a_url_pattern_init(
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

/// <https://urlpattern.spec.whatwg.org/#process-protocol-for-init>
fn process_a_protocol_for_init(input: &str, init_type: PatternInitType) -> Fallible<String> {
    // Step 1. Let strippedValue be the given value with a single trailing U+003A (:) removed, if any.
    let stripped_value = input.strip_suffix(':').unwrap_or(input);

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
pub(super) fn canonicalize_a_protocol(value: &str) -> Fallible<String> {
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
pub(super) fn canonicalize_a_username(input: &str) -> String {
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
pub(super) fn canonicalize_a_password(input: &str) -> String {
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
pub(super) fn canonicalize_a_hostname(input: &str) -> Fallible<String> {
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
pub(super) fn canonicalize_a_port(
    port_value: &str,
    protocol_value: Option<&str>,
) -> Fallible<String> {
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
pub(super) fn canonicalize_a_pathname(value: &str) -> String {
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
pub(super) fn canonicalize_an_opaque_pathname(value: &str) -> Fallible<String> {
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
pub(super) fn canonicalize_a_search(value: &str) -> String {
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
pub(super) fn canonicalize_a_hash(value: &str) -> String {
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
pub(super) fn escape_a_pattern_string(input: &str) -> String {
    escape_a_string(input, &['+', '*', '?', ':', '{', '}', '(', ')', '\\'])
}

/// <https://urlpattern.spec.whatwg.org/#escape-a-regexp-string>
pub(super) fn escape_a_regexp_string(input: &str) -> String {
    escape_a_string(
        input,
        &[
            '.', '+', '*', '?', '^', '$', '{', '}', '(', ')', '[', ']', '|', '/', '\\',
        ],
    )
}
