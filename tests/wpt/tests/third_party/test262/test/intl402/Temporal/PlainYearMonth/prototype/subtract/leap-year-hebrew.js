// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check constraining days due to leap years (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// Adar I (M05L) in common years will be constrained to Adar (M06).
// See also leap-months-hebrew.js

const calendar = "hebrew";
const options = { overflow: "reject" };

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);

const adarI = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M05L", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  adarI.subtract(years1),
  5783, 6, "M06", "Adding 1 year to Adar I constrains to Adar",
  "am", 5783, null);
assert.throws(RangeError, function () {
  adarI.subtract(years1, options);
}, "Adding 1 year to Adar I rejects because the month would be constrained");

TemporalHelpers.assertPlainYearMonth(
  adarI.subtract(years1n),
  5781, 6, "M06", "Subtracting 1 year from Adar I constrains to Adar",
  "am", 5781, null);
assert.throws(RangeError, function () {
  adarI.subtract(years1n, options);
}, "Subtracting 1 year from Adar I rejects because the month would be constrained");
