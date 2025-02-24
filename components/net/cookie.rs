/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! <http://tools.ietf.org/html/rfc6265>

use std::borrow::ToOwned;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::SystemTime;

use cookie::Cookie;
use net_traits::pub_domains::is_pub_domain;
use net_traits::CookieSource;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;

/// A stored cookie that wraps the definition in cookie-rs. This is used to implement
/// various behaviours defined in the spec that rely on an associated request URL,
/// which cookie-rs and hyper's header parsing do not support.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServoCookie {
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    pub cookie: Cookie<'static>,
    pub host_only: bool,
    pub persistent: bool,
    pub creation_time: SystemTime,
    pub last_access: SystemTime,
    pub expiry_time: Option<SystemTime>,
}

impl ServoCookie {
    pub fn from_cookie_string(
        cookie_str: String,
        request: &ServoUrl,
        source: CookieSource,
    ) -> Option<ServoCookie> {
        let cookie = Cookie::parse(cookie_str).ok()?;
        ServoCookie::new_wrapped(cookie, request, source)
    }

    /// Steps 6-22 from <https://www.ietf.org/archive/id/draft-ietf-httpbis-rfc6265bis-15.html#name-storage-model>
    pub fn new_wrapped(
        mut cookie: Cookie<'static>,
        request: &ServoUrl,
        source: CookieSource,
    ) -> Option<ServoCookie> {
        let persistent;
        let expiry_time;

        // Step 6. If the cookie-attribute-list contains an attribute with an attribute-name of "Max-Age":
        if let Some(max_age) = cookie.max_age() {
            // 1. Set the cookie's persistent-flag to true.
            persistent = true;

            // 2. Set the cookie's expiry-time to attribute-value of the last
            // attribute in the cookie-attribute-list with an attribute-name of "Max-Age".
            expiry_time = Some(SystemTime::now() + max_age);
        }
        // Otherwise, if the cookie-attribute-list contains an attribute with an attribute-name of "Expires":
        else if let Some(date_time) = cookie.expires_datetime() {
            // 1. Set the cookie's persistent-flag to true.
            persistent = true;

            // 2. Set the cookie's expiry-time to attribute-value of the last attribute in the
            // cookie-attribute-list with an attribute-name of "Expires".
            expiry_time = Some(date_time.into());
        }
        //  Otherwise:
        else {
            // 1. Set the cookie's persistent-flag to false.
            persistent = false;

            // 2. Set the cookie's expiry-time to the latest representable date.
            expiry_time = None;
        }

        let url_host = request.host_str().unwrap_or("").to_owned();

        // Step 7. If the cookie-attribute-list contains an attribute with an attribute-name of "Domain":
        let mut domain = if let Some(domain) = cookie.domain() {
            // 1. Let the domain-attribute be the attribute-value of the last attribute in the
            // cookie-attribute-list [..]
            // NOTE: This is done by the cookie crate
            domain.to_owned()
        }
        // Otherwise:
        else {
            // 1. Let the domain-attribute be the empty string.
            String::new()
        };

        // TODO Step 8. If the domain-attribute contains a character that is not in the range of [USASCII] characters,
        // abort these steps and ignore the cookie entirely.
        // NOTE: (is this done by the cookies crate?)

        // Step 9. If the user agent is configured to reject "public suffixes" and the domain-attribute
        // is a public suffix:
        if is_pub_domain(&domain) {
            // 1. If the domain-attribute is identical to the canonicalized request-host:
            if domain == url_host {
                // 1. Let the domain-attribute be the empty string.
                domain = String::new();
            }
            //  Otherwise:
            else {
                // 1.Abort these steps and ignore the cookie entirely.
                return None;
            }
        }

        // Step 10. If the domain-attribute is non-empty:
        let host_only;
        if !domain.is_empty() {
            // 1. If the canonicalized request-host does not domain-match the domain-attribute:
            if !ServoCookie::domain_match(&url_host, &domain) {
                // 1. Abort these steps and ignore the cookie entirely.
                return None;
            } else {
                // 1. Set the cookie's host-only-flag to false.
                host_only = false;

                // 2. Set the cookie's domain to the domain-attribute.
                cookie.set_domain(domain);
            }
        }
        // Otherwise:
        else {
            // 1. Set the cookie's host-only-flag to true.
            host_only = true;

            // 2. Set the cookie's domain to the canonicalized request-host.
            cookie.set_domain(url_host);
        };

        // Step 11. If the cookie-attribute-list contains an attribute with an attribute-name of "Path",
        // set the cookie's path to attribute-value of the last attribute in the cookie-attribute-list
        // with both an attribute-name of "Path" and an attribute-value whose length is no more than 1024 octets.
        // Otherwise, set the cookie's path to the default-path of the request-uri.
        let mut has_path_specified = true;
        let mut path = cookie
            .path()
            .unwrap_or_else(|| {
                has_path_specified = false;
                ""
            })
            .to_owned();
        // TODO: Why do we do this?
        if !path.starts_with('/') {
            path = ServoCookie::default_path(request.path()).to_string();
        }
        cookie.set_path(path);

        // Step 12. If the cookie-attribute-list contains an attribute with an attribute-name of "Secure",
        // set the cookie's secure-only-flag to true. Otherwise, set the cookie's secure-only-flag to false.
        let secure_only = cookie.secure().unwrap_or(false);

        // Step 13. If the request-uri does not denote a "secure" connection (as defined by the user agent),
        // and the cookie's secure-only-flag is true, then abort these steps and ignore the cookie entirely.
        if secure_only && !request.is_secure_scheme() {
            return None;
        }

        // Step 14. If the cookie-attribute-list contains an attribute with an attribute-name of "HttpOnly",
        // set the cookie's http-only-flag to true. Otherwise, set the cookie's http-only-flag to false.
        let http_only = cookie.http_only().unwrap_or(false);

        // Step 15. If the cookie was received from a "non-HTTP" API and the cookie's
        // http-only-flag is true, abort these steps and ignore the cookie entirely.
        if http_only && source == CookieSource::NonHTTP {
            return None;
        }

        // TODO: Step 16, Ignore cookies from insecure request uris based on existing cookies

        // TODO: Steps 17-19, same-site-flag

        // Step 20. If the cookie-name begins with a case-insensitive match for the string "__Secure-",
        // abort these steps and ignore the cookie entirely unless the cookie's secure-only-flag is true.
        let has_case_insensitive_prefix = |value: &str, prefix: &str| {
            value
                .get(..prefix.len())
                .is_some_and(|p| p.eq_ignore_ascii_case(prefix))
        };
        if has_case_insensitive_prefix(cookie.name(), "__Secure-") &&
            !cookie.secure().unwrap_or(false)
        {
            return None;
        }

        // Step 21. If the cookie-name begins with a case-insensitive match for the string "__Host-",
        // abort these steps and ignore the cookie entirely unless the cookie meets all the following criteria:
        if has_case_insensitive_prefix(cookie.name(), "__Host-") {
            // 1. The cookie's secure-only-flag is true.
            if !secure_only {
                return None;
            }

            // 2. The cookie's host-only-flag is true.
            if !host_only {
                return None;
            }

            // 3. The cookie-attribute-list contains an attribute with an attribute-name of "Path",
            // and the cookie's path is /.
            #[allow(clippy::nonminimal_bool)]
            if !has_path_specified || !cookie.path().is_some_and(|path| path == "/") {
                return None;
            }
        }

        // Step 22. If the cookie-name is empty and either of the following conditions are true,
        // abort these steps and ignore the cookie entirely:
        if cookie.name().is_empty() {
            // 1. the cookie-value begins with a case-insensitive match for the string "__Secure-"
            if has_case_insensitive_prefix(cookie.value(), "__Secure-") {
                return None;
            }

            // 2. the cookie-value begins with a case-insensitive match for the string "__Host-"
            if has_case_insensitive_prefix(cookie.value(), "__Host-") {
                return None;
            }
        }

        Some(ServoCookie {
            cookie,
            host_only,
            persistent,
            creation_time: SystemTime::now(),
            last_access: SystemTime::now(),
            expiry_time,
        })
    }

    pub fn touch(&mut self) {
        self.last_access = SystemTime::now();
    }

    pub fn set_expiry_time_in_past(&mut self) {
        self.expiry_time = Some(SystemTime::UNIX_EPOCH)
    }

    /// <http://tools.ietf.org/html/rfc6265#section-5.1.4>
    pub fn default_path(request_path: &str) -> &str {
        // Step 2
        if !request_path.starts_with('/') {
            return "/";
        }

        // Step 3
        let rightmost_slash_idx = request_path.rfind('/').unwrap();
        if rightmost_slash_idx == 0 {
            // There's only one slash; it's the first character
            return "/";
        }

        // Step 4
        &request_path[..rightmost_slash_idx]
    }

    /// <http://tools.ietf.org/html/rfc6265#section-5.1.4>
    pub fn path_match(request_path: &str, cookie_path: &str) -> bool {
        // A request-path path-matches a given cookie-path if at least one of
        // the following conditions holds:

        // The cookie-path and the request-path are identical.
        request_path == cookie_path ||
            (request_path.starts_with(cookie_path) &&
                (
                    // The cookie-path is a prefix of the request-path, and the last
                    // character of the cookie-path is %x2F ("/").
                    cookie_path.ends_with('/') ||
            // The cookie-path is a prefix of the request-path, and the first
            // character of the request-path that is not included in the cookie-
            // path is a %x2F ("/") character.
            request_path[cookie_path.len()..].starts_with('/')
                ))
    }

    /// <http://tools.ietf.org/html/rfc6265#section-5.1.3>
    pub fn domain_match(string: &str, domain_string: &str) -> bool {
        let string = &string.to_lowercase();
        let domain_string = &domain_string.to_lowercase();

        string == domain_string ||
            (string.ends_with(domain_string) &&
                string.as_bytes()[string.len() - domain_string.len() - 1] == b'.' &&
                string.parse::<Ipv4Addr>().is_err() &&
                string.parse::<Ipv6Addr>().is_err())
    }

    /// <http://tools.ietf.org/html/rfc6265#section-5.4> step 1
    pub fn appropriate_for_url(&self, url: &ServoUrl, source: CookieSource) -> bool {
        let domain = url.host_str();
        if self.host_only {
            if self.cookie.domain() != domain {
                return false;
            }
        } else if let (Some(domain), Some(cookie_domain)) = (domain, &self.cookie.domain()) {
            if !ServoCookie::domain_match(domain, cookie_domain) {
                return false;
            }
        }

        if let Some(cookie_path) = self.cookie.path() {
            if !ServoCookie::path_match(url.path(), cookie_path) {
                return false;
            }
        }

        if self.cookie.secure().unwrap_or(false) && !url.is_secure_scheme() {
            return false;
        }
        if self.cookie.http_only().unwrap_or(false) && source == CookieSource::NonHTTP {
            return false;
        }

        true
    }
}
