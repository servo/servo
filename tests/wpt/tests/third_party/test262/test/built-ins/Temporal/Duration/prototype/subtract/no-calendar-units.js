// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: >
  Throws if either the receiver or the argument is a duration with nonzero
  calendar units
features: [Temporal]
---*/

const blank = new Temporal.Duration();

const withYears = new Temporal.Duration(1);
assert.throws(RangeError, () => withYears.subtract(blank), "should not subtract from receiver with years");

const withMonths = new Temporal.Duration(0, 1);
assert.throws(RangeError, () => withMonths.subtract(blank), "should not subtract from receiver with months");

const withWeeks = new Temporal.Duration(0, 0, 1);
assert.throws(RangeError, () => withWeeks.subtract(blank), "should not subtract from receiver with weeks");

const ok = new Temporal.Duration(0, 0, 0, 1);

assert.throws(RangeError, () => ok.subtract(withYears), "should not subtract duration with years");
assert.throws(RangeError, () => ok.subtract(withMonths), "should not subtract duration with months");
assert.throws(RangeError, () => ok.subtract(withWeeks), "should not subtract duration with weeks");

assert.throws(RangeError, () => ok.subtract({ years: 1 }), "should not subtract property bag with years");
assert.throws(RangeError, () => ok.subtract({ months: 1 }), "should not subtract property bag with months");
assert.throws(RangeError, () => ok.subtract({ weeks: 1 }), "should not subtract property bag with weeks");

assert.throws(RangeError, () => ok.subtract('P1Y'), "should not subtract string with years");
assert.throws(RangeError, () => ok.subtract('P1M'), "should not subtract string with months");
assert.throws(RangeError, () => ok.subtract('P1W'), "should not subtract string with weeks");
