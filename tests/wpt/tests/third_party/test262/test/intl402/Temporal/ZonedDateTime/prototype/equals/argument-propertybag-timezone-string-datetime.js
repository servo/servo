// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Conversion of ISO date-time strings to time zone IDs (with IANA time zones)
features: [Temporal]
---*/

const expectedTimeZone = "America/Vancouver";
const instance = new Temporal.ZonedDateTime(0n, expectedTimeZone);
let timeZone = "2021-08-19T17:30[America/Vancouver]";
assert(instance.equals({ year: 1969, month: 12, day: 31, hour: 16, timeZone }), "date-time + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30Z[America/Vancouver]";
assert(instance.equals({ year: 1969, month: 12, day: 31, hour: 16, timeZone }), "date-time + Z + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30-07:00[America/Vancouver]";
assert(instance.equals({ year: 1969, month: 12, day: 31, hour: 16, timeZone }), "date-time + offset + IANA annotation is the IANA time zone");
