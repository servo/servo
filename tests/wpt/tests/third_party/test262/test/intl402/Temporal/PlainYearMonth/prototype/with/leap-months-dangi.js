// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Check constraining leap months when year changes in dangi calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";
const options = { overflow: "reject" };
const leapMonth = Temporal.PlainYearMonth.from({ year: 2017, monthCode: "M05L", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  leapMonth.with({ year: 2009 }, options),
  2009, 6, "M05L", "month not constrained when moving to another leap year with M05L",
  undefined, undefined, null);

TemporalHelpers.assertPlainYearMonth(
  leapMonth.with({ year: 2020 }),
  2020, 6, "M05", "month constrained when moving to another leap year without M05L",
  undefined, undefined, null);

assert.throws(RangeError, function () {
  leapMonth.with({ year: 2020 }, options);
}, "reject when moving to another leap year without M05L");

TemporalHelpers.assertPlainYearMonth(
  leapMonth.with({ year: 2024 }),
  2024, 5, "M05", "month constrained when moving to a common year",
  undefined, undefined, null);

assert.throws(RangeError, function () {
  leapMonth.with({ year: 2024 }, options);
}, "reject when moving to a common year");
