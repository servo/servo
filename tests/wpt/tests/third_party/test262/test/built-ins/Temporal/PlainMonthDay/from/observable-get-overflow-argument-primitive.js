// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: overflow property is extracted with string argument.
info: |
    1. Perform ? ToTemporalOverflow(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
];

let actual = [];
const options = TemporalHelpers.propertyBagObserver(actual, { overflow: "reject" }, "options");

const result = Temporal.PlainMonthDay.from("05-17", options);
assert.compareArray(actual, expected, "Successful call");
TemporalHelpers.assertPlainMonthDay(result, "M05", 17);

actual.splice(0);  // empty it for the next check

assert.throws(TypeError, () => Temporal.PlainMonthDay.from(7, options));
assert.compareArray(actual, [], "Failing call before options is processed");

actual.splice(0);

assert.throws(RangeError, () => Temporal.PlainMonthDay.from({ monthCode: "M02", day: 30 }, options));
assert.compareArray(actual, expected, "Failing call after options is processed");
