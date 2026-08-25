// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: String as first argument is equivalent to options bag with unit option
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.Duration(0, 0, 0, 4, 5, 6, 7, 987, 654, 321);
const validUnits = [
  "day",
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
validUnits.forEach((unit) => {
  const full = instance.total({ unit });
  const shorthand = instance.total(unit);
  assert.sameValue(shorthand, full, `"${unit}" as first argument to total is equivalent to options bag`);
});
