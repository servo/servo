// Copyright (C) 2025 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: >
  RangeError thrown if an invalid ISO string (or syntactically valid ISO string
  that is not supported) is used as a PlainTime
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  // invalid ISO strings:
  "",
  "invalid iso8601",
  "00:00Z",
  "Z",
  "25:00:00Z",
  "01:60:00Z",
  "01:60:61Z",
  "00:00Zjunk",
  "00:00:00Zjunk",
  "00:00:00.000000000Zjunk",
  "00:00:00+00:00junk",
  "00:00:00+00:00[UTC]junk",
  "00:00:00+00:00[UTC][u-ca=iso8601]junk",
  "001Z",
  "01:001Z",
  "01:01:001Z",
  "00:00-24:00",
  "00:00+24:00",
  // "00:0000" is invalid (the hour/minute and minute/second separator
  // or lack thereof needs to match).
  "00:00:00+00:0000",
  "0000:00",
  "00:0000",
  "00:00:00+0000:00",
];
for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => Temporal.PlainTime.from(arg),
    `"${arg}" should not be a valid ISO string for a PlainTime`
  );
}
