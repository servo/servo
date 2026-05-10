// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Possibly throw if overflow is reject
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.PlainDateTime.from({year: 2019, month: 1, day: 31}, {overflow: "reject"}),
  2019, 1, "M01", 31, 0, 0, 0, 0, 0, 0,
  "overflow reject, acceptable argument"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from({year: 2019, month: 1, day: 32}, {overflow: "reject"}),
  "overflow reject, unacceptable argument"
);
