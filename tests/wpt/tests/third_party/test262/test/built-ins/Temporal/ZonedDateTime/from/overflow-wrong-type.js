// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-totemporalzoneddatetime steps 2–3:
      2. If Type(_item_) is Object, then
        ...
        j. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
      3. Else,
        a. Perform ? ToTemporalOverflow(_options_).
    sec-temporal.zoneddatetime.from steps 2–3:
      2. If Type(_item_) is Object and _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        a. Perform ? ToTemporalOverflow(_options_).
        ...
        d. Return ...
      3. Return ? ToTemporalZonedDateTime(_item_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const validValues = [
  new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC"),
  "2001-09-09T01:46:40.987654321+00:00[UTC]",
];
validValues.forEach((value) => TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => Temporal.ZonedDateTime.from(value, { overflow }),
  (result, descr) => assert.sameValue(result.epochNanoseconds, 1_000_000_000_987_654_321n, descr),
));

// See TemporalHelpers.checkStringOptionWrongType(); this code path has
// different expectations for observable calls
const propertyBag = { year: 2001, month: 9, day: 9, hour: 1, minute: 46, second: 40, timeZone: "UTC" };

assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: null }), "null");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: true }), "true");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: false }), "false");
assert.throws(TypeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: Symbol() }), "symbol");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: 2n }), "bigint");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: 2 }), "number");
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(propertyBag, { overflow: {} }), "plain object");

// toString property should only be read and converted to a string once, because
// a copied object with the resulting string on it is passed to
// Calendar.dateFromFields().
const expected = [
  "get overflow.toString",
  "call overflow.toString",
];
const actual = [];
const observer = TemporalHelpers.toPrimitiveObserver(actual, "constrain", "overflow");
const result = Temporal.ZonedDateTime.from(propertyBag, { overflow: observer });
assert.sameValue(result.epochNanoseconds, 1_000_000_000_000_000_000n, "object with toString");
assert.compareArray(actual, expected, "order of operations");
