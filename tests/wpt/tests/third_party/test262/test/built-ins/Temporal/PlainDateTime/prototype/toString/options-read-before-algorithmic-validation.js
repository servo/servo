// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.calendarName",
  "get options.calendarName.toString",
  "call options.calendarName.toString",
  "get options.fractionalSecondDigits",
  "get options.fractionalSecondDigits.toString",
  "call options.fractionalSecondDigits.toString",
  "get options.roundingMode",
  "get options.roundingMode.toString",
  "call options.roundingMode.toString",
  "get options.smallestUnit",
  "get options.smallestUnit.toString",
  "call options.smallestUnit.toString",
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  calendarName: "always",
  smallestUnit: "month",
  fractionalSecondDigits: "auto",
  roundingMode: "expand",
}, "options");

const instance = new Temporal.PlainDateTime(2025, 8, 14, 12);

assert.throws(RangeError, function () {
  instance.toString(options);
}, "exception thrown when smallestUnit is a date unit");
assert.compareArray(actual, expected, "all options should be read first");
