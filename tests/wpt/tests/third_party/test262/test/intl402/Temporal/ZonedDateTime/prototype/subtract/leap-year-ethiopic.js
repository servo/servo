// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Check various basic calculations involving leap years (ethiopic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const leapDay = Temporal.ZonedDateTime.from({ year: 2015, monthCode: "M13", day: 6, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

TemporalHelpers.assertPlainDateTime(
  leapDay.subtract(years1).toPlainDateTime(),
  2016, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Adding 1 year to leap day constrains to day 5 of epagomenal month",
  "am", 2016);
assert.throws(RangeError, function () {
  leapDay.subtract(years1, options);
}, "Adding 1 year to leap day rejects");

TemporalHelpers.assertPlainDateTime(
  leapDay.subtract(years1n).toPlainDateTime(),
  2014, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from leap day constrains to day 5 of epagomenal month",
  "am", 2014);
assert.throws(RangeError, function () {
  leapDay.subtract(years1n, options);
}, "Subtracting 1 year from leap day rejects");

TemporalHelpers.assertPlainDateTime(
  leapDay.subtract(years4, options).toPlainDateTime(),
  2019, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Adding 4 years to leap day goes to the next leap day",
  "am", 2019);

TemporalHelpers.assertPlainDateTime(
  leapDay.subtract(years4n, options).toPlainDateTime(),
  2011, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Subtracting 4 years from leap day goes to the previous leap day",
  "am", 2011);
