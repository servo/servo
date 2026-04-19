// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Check constraining leap months when year changes in hebrew calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };
const leapMonth = Temporal.PlainDate.from({ year: 5784, monthCode: "M05L", day: 1, calendar }, options);

TemporalHelpers.assertPlainDate(
  leapMonth.with({ year: 5782 }, options),
  5782, 6, "M05L", 1, "month not constrained when moving to another leap year",
  "am", 5782);

TemporalHelpers.assertPlainDate(
  leapMonth.with({ year: 5783 }),
  5783, 6, "M06", 1, "month constrained when moving to a common year",
  "am", 5783);

assert.throws(RangeError, function () {
  leapMonth.with({ year: 5783 }, options);
}, "reject when moving to a common year");
