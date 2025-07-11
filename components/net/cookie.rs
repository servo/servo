/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! <http://tools.ietf.org/html/rfc6265>

use std::borrow::ToOwned;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::SystemTime;

use cookie::Cookie;
use net_traits::CookieSource;
use net_traits::pub_domains::is_pub_domain;
use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take, take_while_m_n};
use nom::combinator::{opt, recognize};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use time::{Date, Month, OffsetDateTime, Time};

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
        let mut cookie = Cookie::parse(cookie_str.clone()).ok()?;

        // Cookie::parse uses RFC 2616 <http://tools.ietf.org/html/rfc2616#section-3.3.1> to parse
        // cookie expiry date. If it fails to parse the expiry date, try to parse again with
        // less strict algorithm from RFC6265.
        // TODO: We can remove this code and the ServoCookie::parse_date function if cookie-rs
        // library fixes this upstream.
        if cookie.expires_datetime().is_none() {
            let expiry_date_str = cookie_str
                .split(';')
                .filter_map(|key_value| {
                    key_value
                        .find('=')
                        .map(|i| (key_value[..i].trim(), key_value[(i + 1)..].trim()))
                })
                .find_map(|(key, value)| key.eq_ignore_ascii_case("expires").then_some(value));
            if let Some(date_str) = expiry_date_str {
                cookie.set_expires(Self::parse_date(date_str));
            }
        }

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

    /// <https://www.ietf.org/archive/id/draft-ietf-httpbis-rfc6265bis-20.html#name-dates>
    pub fn parse_date(string: &str) -> Option<OffsetDateTime> {
        let string_in_bytes = string.as_bytes();

        // Helper closures
        let parse_ascii_u8 =
            |bytes: &[u8]| -> Option<u8> { std::str::from_utf8(bytes).ok()?.parse::<u8>().ok() };
        let parse_ascii_i32 =
            |bytes: &[u8]| -> Option<i32> { std::str::from_utf8(bytes).ok()?.parse::<i32>().ok() };

        // Step 1. Using the grammar below, divide the cookie-date into date-tokens.
        // *OCTET
        let any_octets = |input| Ok(("".as_bytes(), input));
        // delimiter = %x09 / %x20-2F / %x3B-40 / %x5B-60 / %x7B-7E
        let delimiter: fn(&[u8]) -> IResult<&[u8], u8> = |input| {
            let (input, bytes) = take(1usize)(input)?;
            if matches!(bytes[0], 0x09 | 0x20..=0x2F | 0x3B..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E) {
                Ok((input, bytes[0]))
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }
        };
        // non-delimiter = %x00-08 / %x0A-1F / DIGIT / ":" / ALPHA / %x7F-FF
        let non_delimiter: fn(&[u8]) -> IResult<&[u8], u8> = |input| {
            let (input, bytes) = take(1usize)(input)?;
            if matches!(bytes[0],
                0x00..=0x08 | 0x0A..=0x1F | b'0'..=b'9' | b':' | b'A'..=b'Z' | b'a'..=b'z' | 0x7F..=0xFF)
            {
                Ok((input, bytes[0]))
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }
        };
        // non-digit = %x00-2F / %x3A-FF
        let non_digit: fn(&[u8]) -> IResult<&[u8], u8> = |input| {
            let (input, bytes) = take(1usize)(input)?;
            if matches!(bytes[0], 0x00..=0x2F | 0x3A..=0xFF) {
                Ok((input, bytes[0]))
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )))
            }
        };
        // time-field = 1*2DIGIT
        let time_field = |input| take_while_m_n(1, 2, |byte: u8| byte.is_ascii_digit())(input);
        // hms-time = time-field ":" time-field ":" time-field
        let hms_time = |input| {
            tuple((
                time_field,
                preceded(tag(":"), time_field),
                preceded(tag(":"), time_field),
            ))(input)
        };
        // time = hms-time [ non-digit *OCTET ]
        let time = |input| terminated(hms_time, opt(tuple((non_digit, any_octets))))(input);
        // year = 2*4DIGIT [ non-digit *OCTET ]
        let year = |input| {
            terminated(
                take_while_m_n(2, 4, |byte: u8| byte.is_ascii_digit()),
                opt(tuple((non_digit, any_octets))),
            )(input)
        };
        // month = ( "jan" / "feb" / "mar" / "apr" /
        //           "may" / "jun" / "jul" / "aug" /
        //           "sep" / "oct" / "nov" / "dec" ) *OCTET
        let month = |input| {
            terminated(
                alt((
                    tag_no_case("jan"),
                    tag_no_case("feb"),
                    tag_no_case("mar"),
                    tag_no_case("apr"),
                    tag_no_case("may"),
                    tag_no_case("jun"),
                    tag_no_case("jul"),
                    tag_no_case("aug"),
                    tag_no_case("sep"),
                    tag_no_case("oct"),
                    tag_no_case("nov"),
                    tag_no_case("dec"),
                )),
                any_octets,
            )(input)
        };
        // day-of-month = 1*2DIGIT [ non-digit *OCTET ]
        let day_of_month = |input| {
            terminated(
                take_while_m_n(1, 2, |byte: u8| byte.is_ascii_digit()),
                opt(tuple((non_digit, any_octets))),
            )(input)
        };
        // date-token = 1*non-delimiter
        let date_token = |input| recognize(many1(non_delimiter))(input);
        // date-token-list = date-token *( 1*delimiter date-token )
        let date_token_list = |input| separated_list1(delimiter, date_token)(input);
        // cookie-date = *delimiter date-token-list *delimiter
        let cookie_date =
            |input| delimited(many0(delimiter), date_token_list, many0(delimiter))(input);

        // Step 2. Process each date-token sequentially in the order the date-tokens appear in the cookie-date:
        let mut time_value: Option<(u8, u8, u8)> = None; // Also represents found-time flag.
        let mut day_of_month_value: Option<u8> = None; // Also represents found-day-of-month flag.
        let mut month_value: Option<Month> = None; // Also represents found-month flag.
        let mut year_value: Option<i32> = None; // Also represents found-year flag.

        let (_, date_tokens) = cookie_date(string_in_bytes).ok()?;
        for date_token in date_tokens {
            // Step 2.1. If the found-time flag is not set and the token matches the time production,
            if time_value.is_none() {
                if let Ok((_, result)) = time(date_token) {
                    // set the found-time flag and set the hour-value, minute-value, and
                    // second-value to the numbers denoted by the digits in the date-token,
                    // respectively.
                    if let (Some(hour), Some(minute), Some(second)) = (
                        parse_ascii_u8(result.0),
                        parse_ascii_u8(result.1),
                        parse_ascii_u8(result.2),
                    ) {
                        time_value = Some((hour, minute, second));
                    }
                    // Skip the remaining sub-steps and continue to the next date-token.
                    continue;
                }
            }

            // Step 2.2. If the found-day-of-month flag is not set and the date-token matches the
            // day-of-month production,
            if day_of_month_value.is_none() {
                if let Ok((_, result)) = day_of_month(date_token) {
                    // set the found-day-of-month flag and set the day-of-month-value to the number
                    // denoted by the date-token.
                    day_of_month_value = parse_ascii_u8(result);
                    // Skip the remaining sub-steps and continue to the next date-token.
                    continue;
                }
            }

            // Step 2.3. If the found-month flag is not set and the date-token matches the month production,
            if month_value.is_none() {
                if let Ok((_, result)) = month(date_token) {
                    // set the found-month flag and set the month-value to the month denoted by the date-token.
                    month_value = match std::str::from_utf8(result)
                        .unwrap()
                        .to_ascii_lowercase()
                        .as_str()
                    {
                        "jan" => Some(Month::January),
                        "feb" => Some(Month::February),
                        "mar" => Some(Month::March),
                        "apr" => Some(Month::April),
                        "may" => Some(Month::May),
                        "jun" => Some(Month::June),
                        "jul" => Some(Month::July),
                        "aug" => Some(Month::August),
                        "sep" => Some(Month::September),
                        "oct" => Some(Month::October),
                        "nov" => Some(Month::November),
                        "dec" => Some(Month::December),
                        _ => None,
                    };
                    // Skip the remaining sub-steps and continue to the next date-token.
                    continue;
                }
            }

            // Step 2.4. If the found-year flag is not set and the date-token matches the year production,
            if year_value.is_none() {
                if let Ok((_, result)) = year(date_token) {
                    // set the found-year flag and set the year-value to the number denoted by the date-token.
                    year_value = parse_ascii_i32(result);
                    // Skip the remaining sub-steps and continue to the next date-token.
                    continue;
                }
            }
        }

        // Step 3. If the year-value is greater than or equal to 70 and less than or equal to 99,
        // increment the year-value by 1900.
        if let Some(value) = year_value {
            if (70..=99).contains(&value) {
                year_value = Some(value + 1900);
            }
        }

        // Step 4. If the year-value is greater than or equal to 0 and less than or equal to 69,
        // increment the year-value by 2000.
        if let Some(value) = year_value {
            if (0..=69).contains(&value) {
                year_value = Some(value + 2000);
            }
        }

        // Step 5. Abort these steps and fail to parse the cookie-date if:
        // * at least one of the found-day-of-month, found-month, found-year, or found-time flags is not set,
        if day_of_month_value.is_none() ||
            month_value.is_none() ||
            year_value.is_none() ||
            time_value.is_none()
        {
            return None;
        }
        // * the day-of-month-value is less than 1 or greater than 31,
        if let Some(value) = day_of_month_value {
            if !(1..=31).contains(&value) {
                return None;
            }
        }
        // * the year-value is less than 1601,
        if let Some(value) = year_value {
            if value < 1601 {
                return None;
            }
        }
        // * the hour-value is greater than 23,
        // * the minute-value is greater than 59, or
        // * the second-value is greater than 59.
        if let Some((hour_value, minute_value, second_value)) = time_value {
            if hour_value > 23 || minute_value > 59 || second_value > 59 {
                return None;
            }
        }

        // Step 6. Let the parsed-cookie-date be the date whose day-of-month, month, year, hour,
        // minute, and second (in UTC) are the day-of-month-value, the month-value, the year-value,
        // the hour-value, the minute-value, and the second-value, respectively. If no such date
        // exists, abort these steps and fail to parse the cookie-date.
        let parsed_cookie_date = OffsetDateTime::new_utc(
            Date::from_calendar_date(
                year_value.unwrap(),
                month_value.unwrap(),
                day_of_month_value.unwrap(),
            )
            .ok()?,
            Time::from_hms(
                time_value.unwrap().0,
                time_value.unwrap().1,
                time_value.unwrap().2,
            )
            .ok()?,
        );

        // Step 7. Return the parsed-cookie-date as the result of this algorithm.
        Some(parsed_cookie_date)
    }
}
