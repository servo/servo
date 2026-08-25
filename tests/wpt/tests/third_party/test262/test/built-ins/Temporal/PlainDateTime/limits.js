// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: Checking limits of representable PlainDateTime
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
