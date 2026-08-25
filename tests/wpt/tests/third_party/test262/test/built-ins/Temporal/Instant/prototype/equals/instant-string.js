// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.equals
description: Conversion of ISO date-time strings to Temporal.Instant instances
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

let str = "1970-01-01T00:00";
assert.throws(RangeError, () => instance.equals(str), "bare date-time string is not an instant");
str = "1970-01-01T00:00[UTC]";
assert.throws(RangeError, () => instance.equals(str), "date-time + IANA annotation is not an instant");

str = "1970-01-01T00:00Z";
const result1 = instance.equals(str);
assert.sameValue(result1, true, "date-time + Z preserves exact time");

str = "1970-01-01T00:00+01:00";
const result2 = instance.equals(str);
assert.sameValue(result2, false, "date-time + offset preserves exact time with offset");

str = "1970-01-01T00:00Z[Etc/Ignored]";
const result3 = instance.equals(str);
assert.sameValue(result3, true, "date-time + Z + IANA annotation ignores the IANA annotation");

str = "1970-01-01T00:00+01:00[Etc/Ignored]";
const result4 = instance.equals(str);
assert.sameValue(result4, false, "date-time + offset + IANA annotation ignores the IANA annotation");

str = "1970-01-01T00:00Z[u-ca=hebrew]";
const result6 = instance.equals(str);
assert.sameValue(result6, true, "date-time + Z + Calendar ignores the Calendar");

str = "1970-01-01T00:00+01:00[u-ca=hebrew]";
const result5 = instance.equals(str);
assert.sameValue(result5, false, "date-time + offset + Calendar ignores the Calendar");

str = "1970-01-01T00:00+01:00[Etc/Ignored][u-ca=hebrew]";
const result7 = instance.equals(str);
assert.sameValue(result7, false, "date-time + offset + IANA annotation + Calendar ignores the Calendar and IANA annotation");
