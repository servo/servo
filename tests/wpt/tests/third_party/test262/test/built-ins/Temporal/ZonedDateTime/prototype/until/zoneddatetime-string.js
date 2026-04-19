// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Conversion of ISO date-time strings to Temporal.ZonedDateTime instances
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

let str = "1970-01-01T00:00";
assert.throws(RangeError, () => instance.until(str), "bare date-time string is not a ZonedDateTime");
str = "1970-01-01T00:00Z";
assert.throws(RangeError, () => instance.until(str), "date-time + Z is not a ZonedDateTime");
str = "1970-01-01T00:00+01:00";
assert.throws(RangeError, () => instance.until(str), "date-time + offset is not a ZonedDateTime");

str = "1970-01-01T00:00[+01:00]";
const result1 = instance.until(str);
TemporalHelpers.assertDuration(result1, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, "date-time + IANA annotation preserves wall time in the time zone");

str = "1970-01-01T00:00Z[+01:00]";
const result2 = instance.until(str);
TemporalHelpers.assertDuration(result2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "date-time + Z + IANA annotation preserves exact time in the time zone");

str = "1970-01-01T00:00+01:00[+01:00]";
const result3 = instance.until(str);
TemporalHelpers.assertDuration(result3, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, "date-time + offset + IANA annotation ensures both exact and wall time match");

str = "1970-01-01T00:00-04:15[+01:00]";
assert.throws(RangeError, () => instance.until(str), "date-time + offset + IANA annotation throws if wall time and exact time mismatch");
