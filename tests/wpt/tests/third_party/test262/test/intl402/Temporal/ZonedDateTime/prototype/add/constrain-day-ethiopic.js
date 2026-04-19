// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: >
  Check various basic calculations involving constraining days to the end of the
  epagomenal month (ethiopic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

// Years - see leap-year-ethiopic.js
// Months

const common1230 = Temporal.ZonedDateTime.from({ year: 2014, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const leap0130 = Temporal.ZonedDateTime.from({ year: 2015, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const leap1230 = Temporal.ZonedDateTime.from({ year: 2015, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
const common0130 = Temporal.ZonedDateTime.from({ year: 2016, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const months1 = new Temporal.Duration(0, 1);
const months1n = new Temporal.Duration(0, -1);

TemporalHelpers.assertPlainDateTime(
  common1230.add(months1).toPlainDateTime(),
  2014, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Adding 1 month to last day of Nahase constrains to day 5 of common-year epagomenal month",
  "am", 2014);
assert.throws(RangeError, function () {
  common1230.add(months1, options);
}, "Adding 1 month to last day of Nahase rejects in common year");

TemporalHelpers.assertPlainDateTime(
  leap1230.add(months1).toPlainDateTime(),
  2015, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Adding 1 month to last day of Nahase constrains to day 6 of leap-year epagomenal month",
  "am", 2015);
assert.throws(RangeError, function () {
  leap1230.add(months1, options);
}, "Adding 1 month to last day of Nahase rejects in leap year");

TemporalHelpers.assertPlainDateTime(
  leap0130.add(months1n).toPlainDateTime(),
  2014, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Subtracting 1 month from last day of Maskaram constrains to day 5 of common-year epagomenal month",
  "am", 2014);
assert.throws(RangeError, function () {
  leap0130.add(months1n, options);
}, "Subtracting 1 month from last day of Maskaram rejects in common year");

TemporalHelpers.assertPlainDateTime(
  common0130.add(months1n).toPlainDateTime(),
  2015, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Subtracting 1 month from last day of Maskaram constrains to day 6 of leap-year epagomenal month",
  "am", 2015);
assert.throws(RangeError, function () {
  common0130.add(months1n, options);
}, "Subtracting 1 month from last day of Maskaram rejects in leap year");
