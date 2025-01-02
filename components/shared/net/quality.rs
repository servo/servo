/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//TODO(eijebong): Remove this once typed headers figure out quality
// This is copy pasted from the old hyper headers to avoid hardcoding everything
// (I would probably also make some silly mistakes while migrating...)

use std::{fmt, str};

use http::header::HeaderValue;
use mime::Mime;

/// A quality value, as specified in [RFC7231].
///
/// Quality values are decimal numbers between 0 and 1 (inclusive) with up to 3 fractional digits of precision.
///
/// [RFC7231]: https://tools.ietf.org/html/rfc7231#section-5.3.1
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Quality(u16);

impl Quality {
    /// Creates a quality value from a value between 0 and 1000 inclusive.
    ///
    /// This is semantically divided by 1000 to produce a value between 0 and 1.
    ///
    /// # Panics
    ///
    /// Panics if the value is greater than 1000.
    pub fn from_u16(quality: u16) -> Quality {
        assert!(quality <= 1000);
        Quality(quality)
    }
}

/// A value paired with its "quality" as defined in [RFC7231].
///
/// Quality items are used in content negotiation headers such as `Accept` and `Accept-Encoding`.
///
/// [RFC7231]: https://tools.ietf.org/html/rfc7231#section-5.3
#[derive(Clone, Debug, PartialEq)]
pub struct QualityItem<T> {
    pub item: T,
    pub quality: Quality,
}

impl<T> QualityItem<T> {
    /// Creates a new quality item.
    pub fn new(item: T, quality: Quality) -> QualityItem<T> {
        QualityItem { item, quality }
    }
}

impl<T> fmt::Display for QualityItem<T>
where
    T: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.item, fmt)?;
        match self.quality.0 {
            1000 => Ok(()),
            0 => fmt.write_str(";q=0"),
            mut x => {
                fmt.write_str(";q=0.")?;
                let mut digits = *b"000";
                digits[2] = (x % 10) as u8 + b'0';
                x /= 10;
                digits[1] = (x % 10) as u8 + b'0';
                x /= 10;
                digits[0] = (x % 10) as u8 + b'0';

                let s = str::from_utf8(&digits[..]).unwrap();
                fmt.write_str(s.trim_end_matches('0'))
            },
        }
    }
}

pub fn quality_to_value(q: Vec<QualityItem<Mime>>) -> HeaderValue {
    HeaderValue::from_str(
        &q.iter()
            .map(|q| q.to_string())
            .collect::<Vec<String>>()
            .join(","),
    )
    .unwrap()
}
