// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
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

const result = Temporal.PlainDateTime.from("2021-05-17T12:34:56", options);
assert.compareArray(actual, expected, "Successful call");
TemporalHelpers.assertPlainDateTime(result, 2021, 5, "M05", 17, 12, 34, 56, 0, 0, 0);

actual.splice(0);  // empty it for the next check

assert.throws(TypeError, () => Temporal.PlainDateTime.from(7, options));
assert.compareArray(actual, [], "Failing call before options is processed");

actual.splice(0);

assert.throws(RangeError, () => Temporal.PlainDateTime.from({ year: 2021, month: 2, day: 29 }, options));
assert.compareArray(actual, expected, "Failing call after options is processed");
