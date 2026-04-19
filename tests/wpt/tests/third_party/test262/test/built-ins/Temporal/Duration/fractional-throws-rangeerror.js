// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Temporal.Duration throws a RangeError if any value is fractional
esid: sec-temporal.duration
features: [Temporal]
---*/

const descriptions = [
  'years',
  'months',
  'weeks',
  'days',
  'hours',
  'minutes',
  'seconds',
  'milliseconds',
  'microseconds',
  'nanoseconds'
].map((time) => `Duration constructor throws RangeError with fractional value in the ${time} position`);

assert.throws(RangeError, () => new Temporal.Duration(1.1), descriptions[0]);
assert.throws(RangeError, () => new Temporal.Duration(0, 1.1), descriptions[1]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 1.1), descriptions[2]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 1.1), descriptions[3]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 1.1), descriptions[4]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 1.1), descriptions[5]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 1.1), descriptions[6]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 1.1), descriptions[7]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 1.1), descriptions[8]);
assert.throws(RangeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 1.1), descriptions[9]);
