// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: round() throws if neither largestUnit nor smallestUnit is given
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const hoursOnly = new Temporal.Duration(0, 0, 0, 0, 1);

[
  {},
  () => {
  },
  { roundingMode: "ceil" }
].forEach(roundTo => {
  assert.throws(RangeError, () => d.round(roundTo));
  assert.throws(RangeError, () => hoursOnly.round(roundTo));
});
