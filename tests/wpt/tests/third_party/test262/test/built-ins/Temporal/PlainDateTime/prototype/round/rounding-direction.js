// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Rounding down is towards the Big Bang, not the epoch or 1 BCE
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(-99, 12, 15, 12, 0, 0, 500);
TemporalHelpers.assertPlainDateTime(
  instance.round({ smallestUnit: "second", roundingMode: "floor" }),
  -99, 12, "M12", 15, 12, 0, 0, 0, 0, 0,
  "Rounding down is towards the Big Bang, not the epoch or 1 BCE (roundingMode floor)"
);
TemporalHelpers.assertPlainDateTime(
  instance.round({ smallestUnit: "second", roundingMode: "trunc" }),
  -99, 12, "M12", 15, 12, 0, 0, 0, 0, 0,
  "Rounding down is towards the Big Bang, not the epoch or 1 BCE (roundingMode trunc)"
);
TemporalHelpers.assertPlainDateTime(
  instance.round({ smallestUnit: "second", roundingMode: "ceil" }),
  -99, 12, "M12", 15, 12, 0, 1, 0, 0, 0,
  "Rounding up is away from the Big Bang, not the epoch or 1 BCE (roundingMode ceil)"
);
TemporalHelpers.assertPlainDateTime(
  instance.round({ smallestUnit: "second", roundingMode: "halfExpand" }),
  -99, 12, "M12", 15, 12, 0, 1, 0, 0, 0,
  "Rounding up is away from the Big Bang, not the epoch or 1 BCE (roundingMode halfExpand)"
);
