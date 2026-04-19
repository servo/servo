// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Check various basic calculations involving leap years (indian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "indian";
const options = { overflow: "reject" };

const leap = Temporal.PlainYearMonth.from({ year: 1946, monthCode: "M01", calendar }, options);

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

TemporalHelpers.assertPlainYearMonth(
  leap.add(years1, options),
  1947, 1, "M01", "Adding 1 year to Chaitra",
  "shaka", 1947, null);

TemporalHelpers.assertPlainYearMonth(
  leap.add(years1n, options),
  1945, 1, "M01", "Subtracting 1 year from Chaitra",
  "shaka", 1945, null);

TemporalHelpers.assertPlainYearMonth(
  leap.add(years4, options),
  1950, 1, "M01", "Adding 4 years to Chaitra",
  "shaka", 1950, null);

TemporalHelpers.assertPlainYearMonth(
  leap.add(years4n, options),
  1942, 1, "M01", "Subtracting 4 years from Chaitra",
  "shaka", 1942, null);
