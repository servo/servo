// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Check constraining leap months when year changes in chinese calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";
const options = { overflow: "reject" };
const leapMonth = Temporal.PlainDateTime.from({ year: 2017, monthCode: "M06L", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  leapMonth.with({ year: 2025 }, options),
  2025, 7, "M06L", 1, 12, 34, 0, 0, 0, 0, "month not constrained when moving to another leap year with M06L");

TemporalHelpers.assertPlainDateTime(
  leapMonth.with({ year: 2020 }),
  2020, 7, "M06", 1, 12, 34, 0, 0, 0, 0, "month constrained when moving to another leap year without M06L");

assert.throws(RangeError, function () {
  leapMonth.with({ year: 2020 }, options);
}, "reject when moving to another leap year without M06L");

TemporalHelpers.assertPlainDateTime(
  leapMonth.with({ year: 2024 }),
  2024, 6, "M06", 1, 12, 34, 0, 0, 0, 0, "month constrained when moving to a common year");

assert.throws(RangeError, function () {
  leapMonth.with({ year: 2024 }, options);
}, "reject when moving to a common year");
