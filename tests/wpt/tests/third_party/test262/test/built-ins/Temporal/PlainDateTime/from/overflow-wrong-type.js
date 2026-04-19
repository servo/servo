// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-totemporaldatetime steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        g. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
      3. Else,
        a. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.plaindatetime.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalDateTime]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        b. Return ...
      3. Return ? ToTemporalDateTime(_item_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.PlainDateTime(2000, 5, 2, 12),
  "2000-05-02T12:00",
];
validValues.forEach((value) => TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => Temporal.PlainDateTime.from(value, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainDateTime(result, 2000, 5, "M05", 2, 12, 0, 0, 0, 0, 0, descr),
));

// See TemporalHelpers.checkStringOptionWrongType(); this code path has
// different expectations for observable calls
const propertyBag = { year: 2000, month: 5, day: 2, hour: 12 };

assert.throws(RangeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: null }), "null");
assert.throws(RangeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: true }), "true");
assert.throws(RangeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: false }), "false");
assert.throws(TypeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: Symbol() }), "symbol");
assert.throws(RangeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: 2 }), "number");
assert.throws(RangeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: 2n }), "bigint");
assert.throws(RangeError, () => Temporal.PlainDateTime.from(propertyBag, { overflow: {} }), "plain object");

// toString property should only be read and converted to a string once, because
// a copied object with the resulting string on it is passed to
// Calendar.dateFromFields().
const expected = [
  "get overflow.toString",
  "call overflow.toString",
];
const actual = [];
const observer = TemporalHelpers.toPrimitiveObserver(actual, "constrain", "overflow");
const result = Temporal.PlainDateTime.from(propertyBag, { overflow: observer });
TemporalHelpers.assertPlainDateTime(result, 2000, 5, "M05", 2, 12, 0, 0, 0, 0, 0, "object with toString");
assert.compareArray(actual, expected, "order of operations");
