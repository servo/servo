// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: Conversion of ISO date-time strings to time zone IDs (with IANA time zones)
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

let timeZone = "2021-08-19T17:30[America/Vancouver]";
const result1 = instance.toZonedDateTimeISO(timeZone);
assert.sameValue(result1.timeZoneId, "America/Vancouver", "date-time + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30Z[America/Vancouver]";
const result2 = instance.toZonedDateTimeISO(timeZone);
assert.sameValue(result2.timeZoneId, "America/Vancouver", "date-time + Z + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30-07:00[America/Vancouver]";
const result3 = instance.toZonedDateTimeISO(timeZone);
assert.sameValue(result3.timeZoneId, "America/Vancouver", "date-time + offset + IANA annotation is the IANA time zone");
