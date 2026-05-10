// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Check various basic calculations involving leap years (indian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "indian";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDate.from({ year: 1946, monthCode: "M01", day: 31, calendar }, options);

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

TemporalHelpers.assertPlainDate(
  leapDay.subtract(years1),
  1947, 1, "M01", 30, "Adding 1 year to leap day constrains to 30 Chaitra",
  "shaka", 1947);
assert.throws(RangeError, function () {
  leapDay.subtract(years1, options);
}, "Adding 1 year to leap day rejects");

TemporalHelpers.assertPlainDate(
  leapDay.subtract(years1n),
  1945, 1, "M01", 30, "Subtracting 1 year from leap day constrains to 30 Chaitra",
  "shaka", 1945);
assert.throws(RangeError, function () {
  leapDay.subtract(years1n, options);
}, "Subtracting 1 year from leap day rejects");

TemporalHelpers.assertPlainDate(
  leapDay.subtract(years4, options),
  1950, 1, "M01", 31, "Adding 4 years to leap day goes to the next leap day",
  "shaka", 1950);

TemporalHelpers.assertPlainDate(
  leapDay.subtract(years4n, options),
  1942, 1, "M01", 31, "Subtracting 4 years from leap day goes to the previous leap day",
  "shaka", 1942);
