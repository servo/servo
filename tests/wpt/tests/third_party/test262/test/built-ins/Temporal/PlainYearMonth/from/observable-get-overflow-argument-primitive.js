// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: overflow property is extracted with string argument.
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

const result = Temporal.PlainYearMonth.from("2021-05", options);
assert.compareArray(actual, expected, "Successful call");
TemporalHelpers.assertPlainYearMonth(result, 2021, 5, "M05");

actual.splice(0);  // empty it for the next check

assert.throws(TypeError, () => Temporal.PlainYearMonth.from(7, options));
assert.compareArray(actual, [], "Failing call before options is processed");

actual.splice(0);

assert.throws(RangeError, () => Temporal.PlainYearMonth.from({ year: 2021, month: 13 }, options));
assert.compareArray(actual, expected, "Failing call after options is processed");
