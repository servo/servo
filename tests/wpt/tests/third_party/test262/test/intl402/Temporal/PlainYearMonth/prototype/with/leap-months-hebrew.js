// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Check constraining leap months when year changes in hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };
const leapMonth = Temporal.PlainYearMonth.from({ year: 5784, monthCode: "M05L", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  leapMonth.with({ year: 5782 }, options),
  5782, 6, "M05L", "month not constrained when moving to another leap year",
  "am", 5782, null);

TemporalHelpers.assertPlainYearMonth(
  leapMonth.with({ year: 5783 }),
  5783, 6, "M06", "month constrained when moving to a common year",
  "am", 5783, null);

assert.throws(RangeError, function () {
  leapMonth.with({ year: 5783 }, options);
}, "reject when moving to a common year");
