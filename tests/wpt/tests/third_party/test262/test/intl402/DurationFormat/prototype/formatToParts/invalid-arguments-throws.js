// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: >
  "formatToParts" basic tests for invalid arguments that should throw TypeError exception.
info: |
  Intl.DurationFormat.prototype.formatToParts(duration)
  (...)
  3. Let record be ? ToDurationRecord(duration)
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat();
const testOptions = [ "years", "months", "weeks", "days", "hours", "minutes", "seconds", "milliseconds", "microseconds", "nanoseconds"];

assert.throws(TypeError, () => { df.formatToParts(undefined) }, "undefined" );
assert.throws(TypeError, () => { df.formatToParts(null) }, "null");
assert.throws(TypeError, () => { df.formatToParts(true) }, "true");
assert.throws(TypeError, () => { df.formatToParts(-12) }, "-12");
assert.throws(TypeError, () => { df.formatToParts(-12n) }, "-12n");
assert.throws(TypeError, () => { df.formatToParts(1) }, "1");
assert.throws(TypeError, () => { df.formatToParts(2n) }, "2n");
assert.throws(TypeError, () => { df.formatToParts({}) }, "plain object");
assert.throws(TypeError, () => { df.formatToParts({ year: 1 }) }, "unsuported property");
assert.throws(TypeError, () => { df.formatToParts({ years: undefined }) }, "supported property set undefined");
assert.throws(TypeError, () => { df.formatToParts(Symbol())}, "symbol");
assert.throws(RangeError, () => { df.formatToParts("bad string")}, "bad string");

testOptions.forEach( option => {
  assert.throws(RangeError, () => { df.formatToParts({ [option]: 2.5 })}, " duration properties must be integers");
});

testOptions.forEach( option => {
  assert.throws(RangeError, () => { df.formatToParts({ [option]: -Infinity })}, " duration properties must be integers");
});
