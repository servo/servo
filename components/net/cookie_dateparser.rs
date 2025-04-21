/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::Infallible;

use nom::Err::Failure;
use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while_m_n, take_while1};
use nom::character::{is_alphabetic, is_digit};
use nom::combinator::{eof, map_res};
use nom::error::{Error, ErrorKind};
use nom::multi::many1;
use nom::sequence::{preceded, terminated, tuple};
use time::{Date, Month, OffsetDateTime, Time};

pub fn extract_expiry(cookie: &str) -> Option<OffsetDateTime> {
    let expires_str = cookie
        .split(';')
        .map(str::trim)
        .find(|part| part.to_lowercase().starts_with("expires="))
        .map(|expires_part| &expires_part[8..])?; // length of "expires="

    parse_cookie_date(expires_str)
}

/// <https://datatracker.ietf.org/doc/html/rfc6265#section-5.1.1>
fn parse_cookie_date(input: &str) -> Option<OffsetDateTime> {
    let mut time_value: Option<(u8, u8, u8)> = None;
    let mut day_of_month_value: Option<u8> = None;
    let mut month_value: Option<u8> = None;
    let mut year_value: Option<u16> = None;
    // Note that the variants `Some` and `None` of the variable `time_value`,
    // `day_of_month_value`, `month_value`, and `year_value` represent
    // "true" and "false" of the boolean flags found-time, found-day-of-month,
    // found-month, and found-year, respectively.

    // Step 1
    // Using the grammar (stated and implemented below this function),
    // divide the cookie-date into date-tokens.
    let tokens = cookie_date(input.as_bytes()).unwrap().1;

    // Step 2
    // Process each date-token sequentially in the order the date-tokens
    // appear in the cookie-date:
    for token in tokens {
        // Step 2.1
        // If the found-time flag is not set and the token matches the
        // time production, set the found-time flag and set the hour-
        // value, minute-value, and second-value to the numbers denoted
        // by the digits in the date-token, respectively.  Skip the
        // remaining sub-steps and continue to the next date-token.
        if time_value.is_none() {
            if let Ok(result) = time(token) {
                time_value = Some(result.1);
                continue;
            }
        }

        // Step 2.2
        // If the found-day-of-month flag is not set and the date-token
        // matches the day-of-month production, set the found-day-of-
        // month flag and set the day-of-month-value to the number
        // denoted by the date-token.  Skip the remaining sub-steps and
        // continue to the next date-token.
        if day_of_month_value.is_none() {
            if let Ok(result) = day_of_month(token) {
                day_of_month_value = Some(result.1);
                continue;
            }
        }

        // Step 2.3
        // If the found-month flag is not set and the date-token matches
        // the month production, set the found-month flag and set the
        // month-value to the month denoted by the date-token.  Skip the
        // remaining sub-steps and continue to the next date-token.
        if month_value.is_none() {
            if let Ok(result) = month(token) {
                month_value = Some(result.1);
                // TODO
                continue;
            }
        }

        // Step 2.4
        // If the found-year flag is not set and the date-token matches
        // the year production, set the found-year flag and set the
        // year-value to the number denoted by the date-token.  Skip the
        // remaining sub-steps and continue to the next date-token.
        if year_value.is_none() {
            if let Ok(result) = year(token) {
                year_value = Some(result.1);
                continue;
            }
        }
    }

    // Step 3
    // If the year-value is greater than or equal to 70 and less than or
    // equal to 99, increment the year-value by 1900.
    if let Some(value) = year_value {
        if (70..=99).contains(&value) {
            year_value = Some(value + 1900);
        }
    }

    // Step 4
    // If the year-value is greater than or equal to 0 and less than or
    // equal to 69, increment the year-value by 2000.
    if let Some(value) = year_value {
        if value <= 69 {
            year_value = Some(value + 2000);
        }
    }

    // Step 5
    // Abort these steps and fail to parse the cookie-date if:
    // *  at least one of the found-day-of-month, found-month, found-
    //    year, or found-time flags is not set,
    if day_of_month_value.is_none() ||
        month_value.is_none() ||
        year_value.is_none() ||
        time_value.is_none()
    {
        return None;
    }
    // *  the day-of-month-value is less than 1 or greater than 31,
    if let Some(value) = day_of_month_value {
        if !(1..=31).contains(&value) {
            return None;
        }
    }
    // *  the year-value is less than 1601,
    if let Some(value) = year_value {
        if value < 1601 {
            return None;
        }
    }
    // *  the hour-value is greater than 23,
    // *  the minute-value is greater than 59, or
    // *  the second-value is greater than 59.
    if let Some((hour_value, minute_value, second_value)) = time_value {
        if hour_value > 23 || minute_value > 59 || second_value > 59 {
            return None;
        }
    }

    // Step 6
    // Let the parsed-cookie-date be the date whose day-of-month, month,
    // year, hour, minute, and second (in UTC) are the day-of-month-
    // value, the month-value, the year-value, the hour-value, the
    // minute-value, and the second-value, respectively.  If no such
    // date exists, abort these steps and fail to parse the cookie-date.
    let parsed_cookie_date = OffsetDateTime::new_utc(
        Date::from_calendar_date(
            year_value.unwrap() as i32,
            Month::try_from(month_value.unwrap()).ok()?,
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

    // Step 7
    // Return the parsed-cookie-date as the result of this algorithm.
    Some(parsed_cookie_date)
}

// cookie-date = *delimiter date-token-list *delimiter
fn cookie_date(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    map_res(
        tuple((
            take_while(delimiter),
            date_token_list,
            take_while(delimiter),
        )),
        |(_, token_list, _)| -> Result<Vec<&[u8]>, Error<&[u8]>> { Ok(token_list) },
    )(input)
}

// date-token-list = date-token *( 1*delimiter date-token )
fn date_token_list(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    map_res(
        tuple((
            date_token,
            many1(preceded(take_while1(delimiter), date_token)),
        )),
        |(first_token, rest)| -> Result<Vec<&[u8]>, std::convert::Infallible> {
            let mut tokens = vec![first_token];
            tokens.extend(rest);
            Ok(tokens)
        },
    )(input)
}

// date-token = 1*non-delimiter
fn date_token(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(non_delimiter)(input)
}

// delimiter = %x09 / %x20-2F / %x3B-40 / %x5B-60 / %x7B-7E
fn delimiter(byte: u8) -> bool {
    byte == 0x09 ||
        (0x20..0x2F).contains(&byte) ||
        (0x3B..0x40).contains(&byte) ||
        (0x5B..0x60).contains(&byte) ||
        (0x7B..0x7E).contains(&byte)
}

// non-delimiter = %x00-08 / %x0A-1F / DIGIT / ":" / ALPHA / %x7F-FF
fn non_delimiter(byte: u8) -> bool {
    (0x00..0x08).contains(&byte) ||
        (0x0A..0x1F).contains(&byte) ||
        is_digit(byte) ||
        byte == b':' ||
        is_alphabetic(byte) ||
        (0x7F..0xFF).contains(&byte)
}

// non-digit = %x00-2F / %x3A-FF
fn non_digit(byte: u8) -> bool {
    (0x00..0x2F).contains(&byte) || (0x3A..0xFF).contains(&byte)
}

// day-of-month = 1*2DIGIT ( non-digit *OCTET )
fn day_of_month(input: &[u8]) -> IResult<&[u8], u8> {
    let (trailing, parsed_day_of_month) = map_res(take_while_m_n(1, 2, is_digit), |day| {
        std::str::from_utf8(day)
            .ok()
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| Failure(Error::new(input, ErrorKind::Digit)))
    })(input)?;

    alt((
        eof,
        terminated(take_while_m_n(1, 1, non_digit), take_while(|_| true)),
    ))(trailing)?;
    // Remark: The grammar in RFC6265 requires the token "day-of-month" ends
    // with a non-digit followed by zero or more arbitrary bytes. It is useful
    // to reject the ambiguous cases where the two-digit day is followed by
    // another digit. For example:
    //
    //     123 -> Reject
    //
    // However, it is too strict since it means the grammar rejects the
    // token with two digits only. For example:
    //
    //     12  -> Reject
    //
    // The "eof" above is added to relax the grammar which now allows two
    // digits without any trailing bytes.

    Ok((trailing, parsed_day_of_month))
}

// month = ( "jan" / "feb" / "mar" / "apr" /
//           "may" / "jun" / "jul" / "aug" /
//           "sep" / "oct" / "nov" / "dec" ) *OCTET
fn month(input: &[u8]) -> IResult<&[u8], u8> {
    let (trailing, parsed_month) = map_res(
        alt((
            tag("jan"),
            tag("feb"),
            tag("mar"),
            tag("apr"),
            tag("may"),
            tag("jun"),
            tag("jul"),
            tag("aug"),
            tag("sep"),
            tag("oct"),
            tag("nov"),
            tag("dec"),
        )),
        |month_in_bytes| {
            std::str::from_utf8(month_in_bytes)
                .ok()
                .and_then(|month_in_str: &str| match month_in_str {
                    "jan" => Some(1),
                    "feb" => Some(2),
                    "mar" => Some(3),
                    "apr" => Some(4),
                    "may" => Some(5),
                    "jun" => Some(6),
                    "jul" => Some(7),
                    "aug" => Some(8),
                    "sep" => Some(9),
                    "oct" => Some(10),
                    "nov" => Some(11),
                    "dec" => Some(12),
                    _ => None,
                })
                .ok_or_else(|| Failure(Error::new(input, ErrorKind::MapRes)))
        },
    )(input)?;

    take_while(|_| true)(trailing)?;

    Ok((trailing, parsed_month))
}

// year = 2*4DIGIT ( non-digit *OCTET )
fn year(input: &[u8]) -> IResult<&[u8], u16> {
    let (trailing, parsed_year) = map_res(take_while_m_n(2, 4, is_digit), |year_in_bytes| {
        std::str::from_utf8(year_in_bytes)
            .ok()
            .and_then(|time_field_in_str| time_field_in_str.parse::<u16>().ok())
            .ok_or_else(|| Failure(Error::new(input, ErrorKind::MapRes)))
    })(input)?;

    alt((
        eof,
        terminated(take_while_m_n(1, 1, non_digit), take_while(|_| true)),
    ))(trailing)?;
    // Remark: The grammar in RFC6265 requires the token "year" ends
    // with a non-digit followed by zero or more arbitrary bytes. It is useful
    // to reject the ambiguous cases where the four-digit day is followed by
    // another digit. For example:
    //
    //     20250 -> Reject
    //
    // However, it is too strict since it means the grammar rejects the
    // token with four digits only. For example:
    //
    //     2025  -> Reject
    //
    // The "eof" above is added to relax the grammar which now allows four
    // digits without any trailing bytes.

    Ok((trailing, parsed_year))
}

// time = hms-time ( non-digit *OCTET )
fn time(input: &[u8]) -> IResult<&[u8], (u8, u8, u8)> {
    let (trailing, parsed_time) = map_res(
        hms_time,
        |time| -> Result<(u8, u8, u8), std::convert::Infallible> { Ok(time) },
    )(input)?;

    alt((
        eof,
        terminated(take_while_m_n(1, 1, non_digit), take_while(|_| true)),
    ))(trailing)?;
    // Remark: The grammar in RFC6265 requires the token "time" ends
    // with a non-digit followed by zero or more arbitrary bytes. It is useful
    // to reject the ambiguous cases where the formatted time is followed by
    // another digit. For example:
    //
    //     12:34:567 -> Reject
    //
    // However, it is too strict since it means the grammar rejects the
    // token with formatted time only. For example:
    //
    //     12:34:56  -> Reject
    //
    // The "eof" above is added to relax the grammar which now allows formatted
    // time without any trailing bytes.

    Ok((trailing, parsed_time))
}

// hms-time = time-field ":" time-field ":" time-field
fn hms_time(input: &[u8]) -> IResult<&[u8], (u8, u8, u8)> {
    map_res(
        tuple((time_field, tag(":"), time_field, tag(":"), time_field)),
        |(h, _, m, _, s)| -> Result<(u8, u8, u8), Infallible> { Ok((h, m, s)) },
    )(input)
}

// time-field = 1*2DIGIT
fn time_field(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(
        take_while_m_n(1, 2, is_digit),
        |time_field_in_bytes: &[u8]| {
            std::str::from_utf8(time_field_in_bytes)
                .ok()
                .and_then(|time_field_in_str| time_field_in_str.parse::<u8>().ok())
                .ok_or_else(|| Failure(Error::new(input, ErrorKind::MapRes)))
        },
    )(input)
}
