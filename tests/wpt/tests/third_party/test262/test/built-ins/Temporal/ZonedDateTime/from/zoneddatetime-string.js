// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Conversion of ISO date-time strings to Temporal.ZonedDateTime instances
features: [Temporal]
---*/

let str = "1970-01-01T00:00";
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(str), "bare date-time string is not a ZonedDateTime");
str = "1970-01-01T00:00Z";
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(str), "date-time + Z is not a ZonedDateTime");
str = "1970-01-01T00:00+01:00";
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(str), "date-time + offset is not a ZonedDateTime");

str = "1970-01-01T00:00[+01:00]";
const result1 = Temporal.ZonedDateTime.from(str);
assert.sameValue(result1.epochNanoseconds, -3600_000_000_000n, "date-time + IANA annotation preserves wall time in the time zone");
assert.sameValue(result1.timeZoneId, "+01:00", "IANA annotation is not ignored");

str = "1970-01-01T00:00Z[+01:00]";
const result2 = Temporal.ZonedDateTime.from(str);
assert.sameValue(result2.epochNanoseconds, 0n, "date-time + Z + IANA annotation preserves exact time in the time zone");
assert.sameValue(result2.timeZoneId, "+01:00", "IANA annotation is not ignored");

str = "1970-01-01T00:00+01:00[+01:00]";
const result3 = Temporal.ZonedDateTime.from(str);
assert.sameValue(result3.epochNanoseconds, -3600_000_000_000n, "date-time + offset + IANA annotation ensures both exact and wall time match");
assert.sameValue(result3.timeZoneId, "+01:00", "IANA annotation is not ignored");

str = "1970-01-01T00:00-04:15[+01:00]";
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(str), "date-time + offset + IANA annotation throws if wall time and exact time mismatch");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(str, { offset: "reject" }), "date-time + offset + IANA annotation throws if wall time and exact time mismatch (explicit reject option)");
const result4 = Temporal.ZonedDateTime.from(str, { offset: "ignore" });
assert.sameValue(result4.epochNanoseconds, -3600_000_000_000n, "date-time + wrong offset + IANA annotation preserves wall time in the time zone (offset: ignore option)");
assert.sameValue(result4.timeZoneId, "+01:00", "IANA annotation is not ignored");
