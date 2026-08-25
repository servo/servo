// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Rounding down is towards the Big Bang, not the epoch or 1 BCE
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(-99, 12, 15, 12, 0, 0, 500);
assert.sameValue(
  instance.toString({ smallestUnit: "second", roundingMode: "floor" }),
  "-000099-12-15T12:00:00",
  "Rounding down is towards the Big Bang, not the epoch or 1 BCE"
);
assert.sameValue(
  instance.toString({ smallestUnit: "second", roundingMode: "trunc" }),
  "-000099-12-15T12:00:00",
  "Rounding down is towards the Big Bang, not the epoch or 1 BCE (roundingMode trunc)"
);
assert.sameValue(
  instance.toString({ smallestUnit: "second", roundingMode: "ceil" }),
  "-000099-12-15T12:00:01",
  "Rounding up is away from the Big Bang, not the epoch or 1 BCE (roundingMode ceil)"
);
assert.sameValue(
  instance.toString({ smallestUnit: "second", roundingMode: "halfExpand" }),
  "-000099-12-15T12:00:01",
  "Rounding up is away from the Big Bang, not the epoch or 1 BCE (roundingMode halfExpand)"
);
