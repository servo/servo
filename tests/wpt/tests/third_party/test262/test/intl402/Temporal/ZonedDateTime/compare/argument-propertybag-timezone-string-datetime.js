// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Conversion of ISO date-time strings to Temporal.TimeZone instances (with IANA time zones)
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1588402800_000_000_000n, "America/Vancouver")

let timeZone = "2021-08-19T17:30[America/Vancouver]";
const result1 = Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance);
assert.sameValue(result1, 0, "date-time + IANA annotation is the IANA time zone (first argument)");
const result2 = Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone });
assert.sameValue(result1, 0, "date-time + IANA annotation is the IANA time zone (second argument)");

timeZone = "2021-08-19T17:30Z[America/Vancouver]";
const result3 = Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance);
assert.sameValue(result3, 0, "date-time + Z + IANA annotation is the IANA time zone (first argument)");
const result4 = Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone });
assert.sameValue(result4, 0, "date-time + Z + IANA annotation is the IANA time zone (second argument)");

timeZone = "2021-08-19T17:30-07:00[America/Vancouver]";
const result5 = Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance);
assert.sameValue(result5, 0, "date-time + offset + IANA annotation is the IANA time zone (first argument)");
const result6 = Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone });
assert.sameValue(result6, 0, "date-time + offset + IANA annotation is the IANA time zone (second argument)");
