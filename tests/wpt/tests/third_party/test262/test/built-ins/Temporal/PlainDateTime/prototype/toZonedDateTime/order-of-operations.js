// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Properties on an object passed to toZonedDateTime() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  // ToTemporalDisambiguation
  "get options.disambiguation",
  "get options.disambiguation.toString",
  "call options.disambiguation.toString",
];
const actual = [];

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321, "iso8601");

const options = TemporalHelpers.propertyBagObserver(actual, { disambiguation: "compatible" }, "options");

instance.toZonedDateTime("UTC", options);
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear
