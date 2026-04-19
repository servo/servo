// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Extended format may be used
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const expected = [1976, 11, "M11", 18, 15, 23, 30, 100, 0, 0];

const strs = [
  "1976-11-18T152330.1+00:00",
  "19761118T15:23:30.1+00:00",
  "1976-11-18T15:23:30.1+0000",
  "1976-11-18T152330.1+0000",
  "19761118T15:23:30.1+0000",
  "19761118T152330.1+00:00",
  "19761118T152330.1+0000",
  "+001976-11-18T152330.1+00:00",
  "+0019761118T15:23:30.1+00:00",
  "+001976-11-18T15:23:30.1+0000",
  "+001976-11-18T152330.1+0000",
  "+0019761118T15:23:30.1+0000",
  "+0019761118T152330.1+00:00",
  "+0019761118T152330.1+0000"
];

strs.forEach((s) => {
  TemporalHelpers.assertPlainDateTime(
    Temporal.PlainDateTime.from(s),
    ...expected,
    `mixture of basic and extended format (${s})`
  );
});
