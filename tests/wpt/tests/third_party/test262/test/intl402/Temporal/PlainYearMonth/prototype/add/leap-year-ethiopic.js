// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Check various basic calculations involving leap years (ethiopic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const epagomenalMonth = Temporal.PlainYearMonth.from({ year: 2015, monthCode: "M13", calendar }, options);

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

TemporalHelpers.assertPlainYearMonth(
  epagomenalMonth.add(years1, options),
  2016, 13, "M13", "Adding 1 year to epagomenal month",
  "am", 2016, null);

TemporalHelpers.assertPlainYearMonth(
  epagomenalMonth.add(years1n, options),
  2014, 13, "M13", "Subtracting 1 year from epagomenal month",
  "am", 2014, null);

TemporalHelpers.assertPlainYearMonth(
  epagomenalMonth.add(years4, options),
  2019, 13, "M13", "Adding 4 years to epagomenal month",
  "am", 2019, null);

TemporalHelpers.assertPlainYearMonth(
  epagomenalMonth.add(years4n, options),
  2011, 13, "M13", "Subtracting 4 years from epagomenal month",
  "am", 2011, null);
