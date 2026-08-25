// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Properties on objects passed to round() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.roundingIncrement",
  "get options.roundingIncrement.valueOf",
  "call options.roundingIncrement.valueOf",
  "get options.roundingMode",
  "get options.roundingMode.toString",
  "call options.roundingMode.toString",
  "get options.smallestUnit",
  "get options.smallestUnit.toString",
  "call options.smallestUnit.toString",
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  smallestUnit: "nanoseconds",
  roundingMode: "halfExpand",
  roundingIncrement: 2,
}, "options");

const nextHourOptions = TemporalHelpers.propertyBagObserver(actual, {
  smallestUnit: "hour",
  roundingMode: "ceil",
  roundingIncrement: 1,
}, "options");

const instance = new Temporal.ZonedDateTime(988786472_987_654_321n, /* 2001-05-02T06:54:32.987654321Z */ "UTC");

instance.round(options);
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear
