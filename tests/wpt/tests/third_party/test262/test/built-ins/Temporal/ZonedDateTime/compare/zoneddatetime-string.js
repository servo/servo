// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Conversion of ISO date-time strings to Temporal.ZonedDateTime instances
features: [Temporal]
---*/

const epoch = new Temporal.ZonedDateTime(0n, "UTC");
const hourBefore = new Temporal.ZonedDateTime(-3600_000_000_000n, "UTC");

let str = "1970-01-01T00:00";
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(str, epoch), "bare date-time string is not a ZonedDateTime (first argument)");
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(epoch, str), "bare date-time string is not a ZonedDateTime (second argument)");
str = "1970-01-01T00:00Z";
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(str, epoch), "date-time + Z is not a ZonedDateTime (first argument)");
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(epoch, str), "date-time + Z is not a ZonedDateTime (second argument)");
str = "1970-01-01T00:00+01:00";
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(str, epoch), "date-time + offset is not a ZonedDateTime (first argument)");
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(epoch, str), "date-time + offset is not a ZonedDateTime (second argument)");

str = "1970-01-01T00:00[+01:00]";
const result1 = Temporal.ZonedDateTime.compare(str, hourBefore);
assert.sameValue(result1, 0, "date-time + IANA annotation preserves wall time in the time zone (first argument)");
const result2 = Temporal.ZonedDateTime.compare(hourBefore, str);
assert.sameValue(result2, 0, "date-time + IANA annotation preserves wall time in the time zone (second argument)");

str = "1970-01-01T00:00Z[+01:00]";
const result3 = Temporal.ZonedDateTime.compare(str, epoch);
assert.sameValue(result3, 0, "date-time + Z + IANA annotation preserves exact time in the time zone (first argument)");
const result4 = Temporal.ZonedDateTime.compare(epoch, str);
assert.sameValue(result4, 0, "date-time + Z + IANA annotation preserves exact time in the time zone (second argument)");

str = "1970-01-01T00:00+01:00[+01:00]";
const result5 = Temporal.ZonedDateTime.compare(str, hourBefore);
assert.sameValue(result5, 0, "date-time + offset + IANA annotation ensures both exact and wall time match (first argument)");
const result6 = Temporal.ZonedDateTime.compare(hourBefore, str);
assert.sameValue(result6, 0, "date-time + offset + IANA annotation ensures both exact and wall time match (second argument)");

str = "1970-01-01T00:00-04:15[+01:00]";
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(str, epoch), "date-time + offset + IANA annotation throws if wall time and exact time mismatch (first argument)");
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(epoch, str), "date-time + offset + IANA annotation throws if wall time and exact time mismatch (second argument)");
