// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: months value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);

TemporalHelpers.assertDuration(earlier.until(later, { largestUnit: "months" }),
  0, 13, 0, 0, 0, 0, 0, 0, 0, 0, "largestUnit is months (pos)");
TemporalHelpers.assertDuration(later.until(earlier, { largestUnit: "months" }),
  0, -13, 0, 0, 0, 0, 0, 0, 0, 0, "largestUnit is months (neg)");
