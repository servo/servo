// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Sub-minute offset trailing zeroes allowed in ISO string but not in bracketed offset
features: [Temporal]
---*/

let str = "1970-01-01T01:35:30+01:35:00.000000000[+01:35]";

const result = Temporal.ZonedDateTime.from(str);
assert.sameValue(result.timeZoneId, "+01:35", "ISO offset, sub-minute offset trailing-zeroes");

str = "1970-01-01T01:35:30+01:35:00.000000000[+01:35:00.000000000]";
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from(str),
  "Trailing zeroes not allowed for sub-minute time zone identifiers"
);
