// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Until rounding increments work as expected
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2019, 1);
const later = new Temporal.PlainYearMonth(2021, 9);

const laterSinceYear = earlier.until(later, { smallestUnit: "years", roundingIncrement: 4, roundingMode: "halfExpand" });
TemporalHelpers.assertDuration(laterSinceYear,
  /* years = */ 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, "rounds to an increment of years");

const laterSinceMixed = earlier.until(later, { smallestUnit: "months", roundingIncrement: 5 });
TemporalHelpers.assertDuration(laterSinceMixed,
  /* years = */ 2, /* months = */ 5, 0, 0, 0, 0, 0, 0, 0, 0, "rounds to an increment of months mixed with years");

const laterSinceMonth = earlier.until(later, { largestUnit: "months", smallestUnit: "months", roundingIncrement: 10 });
TemporalHelpers.assertDuration(laterSinceMonth,
  0, /* months = */ 30, 0, 0, 0, 0, 0, 0, 0, 0, "rounds to an increment of pure months");
