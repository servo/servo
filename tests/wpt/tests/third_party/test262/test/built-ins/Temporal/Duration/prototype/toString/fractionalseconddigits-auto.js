// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: auto value for fractionalSecondDigits option
features: [Temporal]
---*/

const wholeSeconds = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7);
const subSeconds = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 650);

const tests = [
  [wholeSeconds, "P1Y2M3W4DT5H6M7S"],
  [subSeconds, "P1Y2M3W4DT5H6M7.98765S"],
];

for (const [duration, expected] of tests) {
  assert.sameValue(duration.toString(), expected, "default is to emit seconds and drop trailing zeroes");
  assert.sameValue(duration.toString({ fractionalSecondDigits: "auto" }), expected, "auto is the default");
}
