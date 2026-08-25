// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Fallback value for largestUnit option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);

TemporalHelpers.assertDuration(later.since(earlier, { largestUnit: undefined }),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (explicit, pos)");
TemporalHelpers.assertDuration(earlier.since(later, { largestUnit: undefined }),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (explicit, neg)");

TemporalHelpers.assertDuration(later.since(earlier, {}),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (implicit, pos)");
TemporalHelpers.assertDuration(earlier.since(later, {}),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (implicit, neg)");

TemporalHelpers.assertDuration(later.since(earlier, () => {}),
  1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (arrow function, pos)");
TemporalHelpers.assertDuration(earlier.since(later, () => {}),
  -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (arrow function, neg)");
