// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check various basic calculations involving leap years (ethiopic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainYearMonth.from({ year: 2015, monthCode: "M13", calendar }, options);

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years4 = new Temporal.Duration(-4);
const years4n = new Temporal.Duration(4);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years1, options),
  2016, 13, "M13", "Adding 1 year to leap day",
  "am", 2016, null);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years1n, options),
  2014, 13, "M13", "Subtracting 1 year from leap day",
  "am", 2014, null);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years4, options),
  2019, 13, "M13", "Adding 4 years to leap day",
  "am", 2019, null);

TemporalHelpers.assertPlainYearMonth(
  leapDay.subtract(years4n, options),
  2011, 13, "M13", "Subtracting 4 years from leap day",
  "am", 2011, null);
