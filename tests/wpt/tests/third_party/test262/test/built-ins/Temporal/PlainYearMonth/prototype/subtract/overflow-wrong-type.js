// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isoyearmonthfromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plainyearmonth.prototype.subtract steps 13–15:
      13. Let _addedDate_ be ? CalendarDateAdd(_calendar_, _date_, _durationToAdd_, _options_).
      14. ...
      15. Return ? YearMonthFromFields(_calendar_, _addedDateFields_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const yearmonth = new Temporal.PlainYearMonth(2000, 5);
const duration = new Temporal.Duration(1, 1);

// See TemporalHelpers.checkStringOptionWrongType(); this code path has
// different expectations for observable calls

assert.throws(RangeError, () => yearmonth.subtract(duration, { overflow: null }), "null");
assert.throws(RangeError, () => yearmonth.subtract(duration, { overflow: true }), "true");
assert.throws(RangeError, () => yearmonth.subtract(duration, { overflow: false }), "false");
assert.throws(TypeError, () => yearmonth.subtract(duration, { overflow: Symbol() }), "symbol");
assert.throws(RangeError, () => yearmonth.subtract(duration, { overflow: 2 }), "number");
assert.throws(RangeError, () => yearmonth.subtract(duration, { overflow: 2n }), "bigint");
assert.throws(RangeError, () => yearmonth.subtract(duration, { overflow: {} }), "plain object");

const expected = [
  "get overflow.toString",
  "call overflow.toString",
];
const actual = [];
const observer = TemporalHelpers.toPrimitiveObserver(actual, "constrain", "overflow");
const result = yearmonth.subtract(duration, { overflow: observer });
TemporalHelpers.assertPlainYearMonth(result, 1999, 4, "M04", "object with toString");
assert.compareArray(actual, expected, "order of operations");
