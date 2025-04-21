/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net::cookie_dateparser::extract_expiry;
use time::error::ComponentRange;
use time::{Date, Month, OffsetDateTime, Time};

#[test]
fn test_extract_expiry() -> Result<(), ComponentRange> {
    assert_eq!(
        extract_expiry("test=1; expires=01 jan 2024 12:59:59 GMT; Path=/"),
        Some(OffsetDateTime::new_utc(
            Date::from_calendar_date(2024, Month::January, 1)?,
            Time::from_hms(12, 59, 59)?,
        ))
    );
    assert_eq!(extract_expiry("test=1; expires=not a date; Path=/"), None);

    Ok(())
}
