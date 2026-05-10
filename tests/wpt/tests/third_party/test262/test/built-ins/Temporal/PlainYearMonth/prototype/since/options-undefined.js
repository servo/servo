// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Verify that undefined options are handled correctly.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2002, 12);

TemporalHelpers.assertDuration(later.since(earlier, undefined),
  2, 7, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (explicit, pos)");
TemporalHelpers.assertDuration(earlier.since(later, undefined),
  -2, -7, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (explicit, neg)");

TemporalHelpers.assertDuration(later.since(earlier),
  2, 7, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (implicit, pos)");
TemporalHelpers.assertDuration(earlier.since(later),
  -2, -7, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (implicit, neg)");

TemporalHelpers.assertDuration(later.since(earlier, () => {}),
  2, 7, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (lambda, pos)");
TemporalHelpers.assertDuration(earlier.since(later, () => {}),
  -2, -7, 0, 0, 0, 0, 0, 0, 0, 0, "default largestUnit is year (lambda, neg)");
