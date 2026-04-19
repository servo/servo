// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Sub-minute offset trailing zeroes allowed in ISO string but not in bracketed offset
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(30_000_000_000n, "+01:35");
let str = "1970-01-01T01:35:30+01:35:00.000000000[+01:35]";

assert.sameValue(
  Temporal.ZonedDateTime.compare(str, datetime),
  0,
  "Time zone determined from bracket name (first argument)"
);
assert.sameValue(
  Temporal.ZonedDateTime.compare(datetime, str),
  0,
  "Time zone determined from bracket name (second argument)"
);

str = "1970-01-01T01:35:30+01:35:00.000000000[+01:35:00.000000000]";
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.compare(str, datetime),
  "Trailing zeroes not allowed for sub-minute time zone identifiers (first argument)"
);
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.compare(datetime, str),
  "Trailing zeroes not allowed for sub-minute time zone identifiers (second argument)"
);
