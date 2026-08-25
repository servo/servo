// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: auto value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);

TemporalHelpers.assertDuration(earlier.until(later, { largestUnit: "auto" }),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "auto largestUnit is year (pos)");
TemporalHelpers.assertDuration(later.until(earlier, { largestUnit: "auto" }),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "auto largestUnit is year (neg)");
