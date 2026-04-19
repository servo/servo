// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: >
  Throws if either the receiver or the argument is a duration with nonzero
  calendar units
features: [Temporal]
---*/

const blank = new Temporal.Duration();

const withYears = new Temporal.Duration(1);
assert.throws(RangeError, () => withYears.add(blank), "should not add to receiver with years");

const withMonths = new Temporal.Duration(0, 1);
assert.throws(RangeError, () => withMonths.add(blank), "should not add to receiver with months");

const withWeeks = new Temporal.Duration(0, 0, 1);
assert.throws(RangeError, () => withWeeks.add(blank), "should not add to receiver with weeks");

const ok = new Temporal.Duration(0, 0, 0, 1);

assert.throws(RangeError, () => ok.add(withYears), "should not add duration with years");
assert.throws(RangeError, () => ok.add(withMonths), "should not add duration with months");
assert.throws(RangeError, () => ok.add(withWeeks), "should not add duration with weeks");

assert.throws(RangeError, () => ok.add({ years: 1 }), "should not add property bag with years");
assert.throws(RangeError, () => ok.add({ months: 1 }), "should not add property bag with months");
assert.throws(RangeError, () => ok.add({ weeks: 1 }), "should not add property bag with weeks");

assert.throws(RangeError, () => ok.add('P1Y'), "should not add string with years");
assert.throws(RangeError, () => ok.add('P1M'), "should not add string with months");
assert.throws(RangeError, () => ok.add('P1W'), "should not add string with weeks");
