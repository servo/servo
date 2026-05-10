// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Sub-minute offset trailing zeroes allowed in ISO string but not in bracketed offset
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "+01:35";
const instance = new Temporal.ZonedDateTime(0n, timeZone);
let str = "1970-01-01T01:35:30+01:35:00.000000000[+01:35]";

const result = instance.since(str);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, -30, 0, 0, 0, "ISO offset, sub-minute offset trailing-zeroes");

str = "1970-01-01T01:35:30+01:35:00.000000000[+01:35:00.000000000]";
assert.throws(
  RangeError,
  () => instance.since(str),
  "Trailing zeroes not allowed for sub-minute time zone identifiers"
);
