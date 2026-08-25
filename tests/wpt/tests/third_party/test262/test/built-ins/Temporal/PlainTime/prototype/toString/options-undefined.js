// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const tests = [
  ["15:23", "15:23:00"],
  ["15:23:30", "15:23:30"],
  ["15:23:30.1234", "15:23:30.1234"],
];

for (const [input, expected] of tests) {
  const time = Temporal.PlainTime.from(input);

  const explicit = time.toString(undefined);
  assert.sameValue(explicit, expected, "default precision is auto and no rounding");

  const implicit = time.toString();
  assert.sameValue(implicit, expected, "default precision is auto and no rounding");

  const lambda = time.toString(() => {});
  assert.sameValue(lambda, expected, "default precision is auto and no rounding");
}
