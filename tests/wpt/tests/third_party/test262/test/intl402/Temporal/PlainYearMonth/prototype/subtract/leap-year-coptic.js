// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check various basic calculations involving leap years (coptic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainYearMonth.from({ year: 1739, monthCode: "M13", calendar }, options);

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years1, options),
  1740, 13, "M13", "Adding 1 year to epagomenal month",
  "am", 1740, null);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years1n, options),
  1738, 13, "M13", "Subtracting 1 year from epagomenal month",
  "am", 1738, null);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years4, options),
  1743, 13, "M13", "Adding 4 years to epagomenal month",
  "am", 1743, null);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years4n, options),
  1735, 13, "M13", "Subtracting 4 years from epagomenal month",
  "am", 1735, null);
