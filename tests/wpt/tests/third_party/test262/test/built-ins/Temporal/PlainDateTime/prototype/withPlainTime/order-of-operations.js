// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.withplaintime
description: Properties on an object passed to withPlainTime() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get fields.hour",
  "get fields.hour.valueOf",
  "call fields.hour.valueOf",
  "get fields.microsecond",
  "get fields.microsecond.valueOf",
  "call fields.microsecond.valueOf",
  "get fields.millisecond",
  "get fields.millisecond.valueOf",
  "call fields.millisecond.valueOf",
  "get fields.minute",
  "get fields.minute.valueOf",
  "call fields.minute.valueOf",
  "get fields.nanosecond",
  "get fields.nanosecond.valueOf",
  "call fields.nanosecond.valueOf",
  "get fields.second",
  "get fields.second.valueOf",
  "call fields.second.valueOf",
];
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  year: 1.7,
  month: 1.7,
  monthCode: "M01",
  day: 1.7,
  hour: 1,
  microsecond: 2,
  millisecond: 3,
  minute: 4,
  nanosecond: 5,
  second: 6,
  calendar: "iso8601",
}, "fields", ["calendar"]);

const pdt = new Temporal.PlainDateTime(2025, 5, 2, 0, 0, 0, 0, 0, 0);

pdt.withPlainTime(fields);
assert.compareArray(actual, expected, "order of operations");
