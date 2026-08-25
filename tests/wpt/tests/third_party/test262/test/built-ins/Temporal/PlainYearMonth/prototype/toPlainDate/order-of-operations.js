// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.toplaindate
description: Properties on an object passed to toPlainDate() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get fields.day",
  "get fields.day.valueOf",
  "call fields.day.valueOf",
];
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  year: 1.7,
  month: 1.7,
  monthCode: "M01",
  day: 1,
  calendar: "iso8601",
}, "fields", ["calendar"]);

const pym = new Temporal.PlainYearMonth(2005, 2);

pym.toPlainDate(fields);
assert.compareArray(actual, expected, "order of operations");
