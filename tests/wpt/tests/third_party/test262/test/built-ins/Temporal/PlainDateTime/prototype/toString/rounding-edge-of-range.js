// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Rounding can cause RangeError at edge of representable range
features: [Temporal]
---*/

const start = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 1);
assert.throws(
  RangeError,
  () => start.toString({ smallestUnit: "second" }),
  "Rounding down can go out of range"
);

const end = new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999);
assert.throws(
  RangeError,
  () => end.toString({ smallestUnit: "second", roundingMode: "halfExpand" }),
  "Rounding up can go out of range"
);
