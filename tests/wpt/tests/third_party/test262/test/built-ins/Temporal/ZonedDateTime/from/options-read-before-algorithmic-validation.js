// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  All options properties are read and cast before any algorithmic validation
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get options.disambiguation",
  "get options.disambiguation.toString",
  "call options.disambiguation.toString",
  "get options.offset",
  "get options.offset.toString",
  "call options.offset.toString",
  "get options.overflow",
  "get options.overflow.toString",
  "call options.overflow.toString",
];
const actual = [];

const options = TemporalHelpers.propertyBagObserver(actual, {
  overflow: "constrain",
  offset: "prefer",
  disambiguation: "compatible",
}, "options");

assert.throws(RangeError, function () {
  Temporal.ZonedDateTime.from({ year: 2025, monthCode: "M08L", day: 14, timeZone: "UTC" }, options);
}, "exception thrown when month code is invalid for calendar");
assert.compareArray(actual, expected, "all options should be read first");
