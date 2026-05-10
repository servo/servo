// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Check constraining days due to leap years (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// Adar I (M05L) has 30 days, and in common years will be constrained to Adar
// (M06) which has 29 days.
// See also leap-months-hebrew.js and constrain-day-hebrew.js.

const calendar = "hebrew";
const options = { overflow: "reject" };

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);

const adarI = Temporal.PlainDate.from({ year: 5782, monthCode: "M05L", day: 30, calendar }, options);

TemporalHelpers.assertPlainDate(
  adarI.subtract(years1),
  5783, 6, "M06", 29, "Adding 1 year to 30 Adar I constrains to 29 Adar",
  "am", 5783);
assert.throws(RangeError, function () {
  adarI.subtract(years1, options);
}, "Adding 1 year to 30 Adar I rejects (either because the month or day would be constrained)");

TemporalHelpers.assertPlainDate(
  adarI.subtract(years1n),
  5781, 6, "M06", 29, "Subtracting 1 year from 30 Adar I constrains to 29 Adar",
  "am", 5781);
assert.throws(RangeError, function () {
  adarI.subtract(years1n, options);
}, "Subtracting 1 year from 30 Adar I rejects (either because the month or day would be constrained)");
