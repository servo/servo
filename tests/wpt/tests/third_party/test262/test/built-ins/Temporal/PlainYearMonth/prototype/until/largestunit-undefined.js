// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Fallback value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);

TemporalHelpers.assertDuration(earlier.until(later, { largestUnit: undefined }),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (explicit, pos)");
TemporalHelpers.assertDuration(later.until(earlier, { largestUnit: undefined }),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (explicit, neg)");

TemporalHelpers.assertDuration(earlier.until(later, {}),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (implicit, pos)");
TemporalHelpers.assertDuration(later.until(earlier, {}),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (implicit, neg)");

TemporalHelpers.assertDuration(earlier.until(later, () => {}),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (arrow function, pos)");
TemporalHelpers.assertDuration(later.until(earlier, () => {}),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (arrow function, neg)");
