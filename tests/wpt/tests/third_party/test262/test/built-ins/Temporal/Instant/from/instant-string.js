// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Conversion of ISO date-time strings to Temporal.Instant instances
features: [Temporal]
---*/

let str = "1970-01-01T00:00";
assert.throws(RangeError, () => Temporal.Instant.from(str), "bare date-time string is not an instant");
str = "1970-01-01T00:00[UTC]";
assert.throws(RangeError, () => Temporal.Instant.from(str), "date-time + IANA annotation is not an instant");

str = "1970-01-01T00:00Z";
const result1 = Temporal.Instant.from(str);
assert.sameValue(result1.epochNanoseconds, 0n, "date-time + Z preserves exact time");

str = "1970-01-01T00:00+01:00";
const result2 = Temporal.Instant.from(str);
assert.sameValue(result2.epochNanoseconds, -3600_000_000_000n, "date-time + offset preserves exact time with offset");

str = "1970-01-01T00:00Z[Etc/Ignored]";
const result3 = Temporal.Instant.from(str);
assert.sameValue(result3.epochNanoseconds, 0n, "date-time + Z + IANA annotation ignores the IANA annotation");

str = "1970-01-01T00:00+01:00[Etc/Ignored]";
const result4 = Temporal.Instant.from(str);
assert.sameValue(result4.epochNanoseconds, -3600_000_000_000n, "date-time + offset + IANA annotation ignores the IANA annotation");

str = "1970-01-01T00:00Z[u-ca=hebrew]";
const result6 = Temporal.Instant.from(str);
assert.sameValue(result6.epochNanoseconds, 0n, "date-time + Z + Calendar ignores the Calendar");

str = "1970-01-01T00:00+01:00[u-ca=hebrew]";
const result7 = Temporal.Instant.from(str);
assert.sameValue(result7.epochNanoseconds, -3600_000_000_000n, "date-time + offset + Calendar ignores the Calendar");

str = "1970-01-01T00:00+01:00[Etc/Ignored][u-ca=hebrew]";
const result8 = Temporal.Instant.from(str);
assert.sameValue(result8.epochNanoseconds, -3600_000_000_000n, "date-time + offset + IANA annotation + Calendar ignores the Calendar and IANA annotation");
