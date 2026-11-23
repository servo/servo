// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: Checking limits of representable PlainDateTime
includes: [temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(
  RangeError,
  () => new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 0),
  "negative year out of bounds"
);
assert.throws(
  RangeError,
  () => new Temporal.PlainDateTime(275760, 9, 14, 0, 0, 0, 0, 0, 0),
  "positive year out of bounds"
);

TemporalHelpers.assertPlainDateTime(
  new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1),
  -271821, 4, "M04", 19, 0, 0, 0, 0, 0, 1,
  "negative boundary"
);
TemporalHelpers.assertPlainDateTime(
  new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999),
  275760, 9, "M09", 13, 23, 59, 59, 999, 999, 999,
  "positive boundary"
);
