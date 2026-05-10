// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.toplaindate
description: Properties on an object passed to toPlainDate() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get fields.year",
  "get fields.year.valueOf",
  "call fields.year.valueOf",
];
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  year: 1.7,
  month: 1.7,
  monthCode: "M01",
  day: 1.7,
  calendar: "iso8601",
}, "fields", ["calendar"]);

const pmd = new Temporal.PlainMonthDay(5, 2);

pmd.toPlainDate(fields);
assert.compareArray(actual, expected, "order of operations");
