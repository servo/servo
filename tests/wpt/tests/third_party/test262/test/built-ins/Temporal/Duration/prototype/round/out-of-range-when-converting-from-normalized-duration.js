// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
    When converting the result from normalized duration form, each duration
    component is turned into a float64-representable integer
features: [Temporal]
---*/

/*
const maxMs = 9_007_199_254_740_991_487;
const maxUs = 9_007_199_254_740_991_475_711;
const maxNs = 9_007_199_254_740_991_463_129_087;
*/

const ms = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* s = */ Number.MAX_SAFE_INTEGER, /* ms = */ 488, 0, 0);
assert.throws(RangeError, () => ms.round({
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
}), "nanoseconds component after balancing as a float64-representable integer is out of range (maximum milliseconds)");

const us = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* s = */ Number.MAX_SAFE_INTEGER, 0, /* us = */ 475_712, 0);
assert.throws(RangeError, () => ms.round({
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
}), "nanoseconds component after balancing as a float64-representable integer is out of range (maximum microseconds)");

const ns = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* s = */ Number.MAX_SAFE_INTEGER, 0, 0, /* ns = */ 463_129_088);
assert.throws(RangeError, () => ns.round({
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
}), "nanoseconds component after balancing as a float64-representable integer is out of range (maximum nanoseconds)");

const msMin = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* s = */ -Number.MAX_SAFE_INTEGER, /* ms = */ -487, 0, 0);
assert.throws(RangeError, () => msMin.round({
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
}), "nanoseconds component after balancing as a float64-representable integer is out of range (minimum milliseconds)");

const usMin = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* s = */ -Number.MAX_SAFE_INTEGER, 0, /* us = */ -475_711, 0);
assert.throws(RangeError, () => usMin.round({
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
}), "nanoseconds component after balancing as a float64-representable integer is out of range (minimum microseconds)");

const nsMin = new Temporal.Duration(0, 0, 0, 0, 0, 0, /* s = */ -Number.MAX_SAFE_INTEGER, 0, 0, /* ns = */ -463_129_088);
assert.throws(RangeError, () => nsMin.round({
  largestUnit: "nanoseconds",
  roundingIncrement: 1,
}), "nanoseconds component after balancing as a float64-representable integer is out of range (minimum nanoseconds)");
