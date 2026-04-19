// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.zoneddatetime.prototype.with step 24:
      24. Let _dateTimeResult_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");

// See TemporalHelpers.checkStringOptionWrongType(); this code path has
// different expectations for observable calls

assert.throws(RangeError, () => datetime.with({ second: 41 }, { overflow: null }), "null");
assert.throws(RangeError, () => datetime.with({ second: 41 }, { overflow: true }), "true");
assert.throws(RangeError, () => datetime.with({ second: 41 }, { overflow: false }), "false");
assert.throws(TypeError, () => datetime.with({ second: 41 }, { overflow: Symbol() }), "symbol");
assert.throws(RangeError, () => datetime.with({ second: 41 }, { overflow: 2 }), "number");
assert.throws(RangeError, () => datetime.with({ second: 41 }, { overflow: 2n }), "bigint");
assert.throws(RangeError, () => datetime.with({ second: 41 }, { overflow: {} }), "plain object");

// toString property should only be read and converted to a string once, because
// a copied object with the resulting string on it is passed to
// Calendar.dateFromFields().
const expected = [
  "get overflow.toString",
  "call overflow.toString",
];
const actual = [];
const observer = TemporalHelpers.toPrimitiveObserver(actual, "constrain", "overflow");
const result = datetime.with({ second: 41 }, { overflow: observer });
assert.sameValue(result.epochNanoseconds, 1_000_000_001_987_654_321n, "object with toString");
assert.compareArray(actual, expected, "order of operations");
