// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Conversion of ISO date-time strings to Temporal.TimeZone instances (with IANA time zones)
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

let timeZone = "2021-08-19T17:30[America/Vancouver]";
const result1 = instance.toString({ timeZone });
assert.sameValue(result1.substr(-6), "-08:00", "date-time + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30Z[America/Vancouver]";
const result2 = instance.toString({ timeZone });
assert.sameValue(result2.substr(-6), "-08:00", "date-time + Z + IANA annotation is the IANA time zone");

timeZone = "2021-08-19T17:30-07:00[America/Vancouver]";
const result3 = instance.toString({ timeZone });
assert.sameValue(result3.substr(-6), "-08:00", "date-time + offset + IANA annotation is the IANA time zone");
