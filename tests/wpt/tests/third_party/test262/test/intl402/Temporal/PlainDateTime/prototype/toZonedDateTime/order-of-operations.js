// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: >
  Properties on an object passed to toZonedDateTime() are accessed in the
  correct order (with disambiguation=reject)
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

const instance = new Temporal.PlainDateTime(2000, 4, 2, 2, 30);
const options = TemporalHelpers.propertyBagObserver(actual, { disambiguation: "reject" }, "options");
assert.throws(RangeError, () => instance.toZonedDateTime("America/Vancouver", options));
assert.compareArray(actual, expected, "order of operations with disambiguation: reject");
actual.splice(0); // clear
