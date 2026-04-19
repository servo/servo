// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Subtracting a datetime in a leap year from a date in a common year should work
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const a = new Temporal.PlainDateTime(2016, 7, 31, 0, 0, 0, 0, 0, 0, 'chinese');
const b = new Temporal.PlainDateTime(2017, 7, 31, 0, 0, 0, 0, 0, 0, 'chinese');
TemporalHelpers.assertDuration(a.until(b, { largestUnit: 'years' }),
  1, 0, 0, 10, 0, 0, 0, 0, 0, 0, "Chinese calendar, year-and-a-bit");

const c = new Temporal.PlainDateTime(1967, 2, 28, 0, 0, 0, 0, 0, 0, 'hebrew');
const d = new Temporal.PlainDateTime(1968, 3, 1, 0, 0, 0, 0, 0, 0, 'hebrew');
TemporalHelpers.assertDuration(c.until(d, { largestUnit: 'years' }),
  0, 12, 0, 13, 0, 0, 0, 0, 0, 0, "Hebrew calendar, year-and-a-bit");

const e = new Temporal.PlainDateTime(2016, 3, 31, 0, 0, 0, 0, 0, 0, 'chinese');
const f = new Temporal.PlainDateTime(2019, 3, 29, 0, 0, 0, 0, 0, 0, 'chinese');
TemporalHelpers.assertDuration(e.until(f, { largestUnit: 'years' }),
  3, 0, 0, 0, 0, 0, 0, 0, 0, 0, "Chinese calendar, 3 years");

const g = new Temporal.PlainDateTime(2019, 5, 1, 0, 0, 0, 0, 0, 0, 'chinese');
const h = new Temporal.PlainDateTime(2020, 4, 19, 0, 0, 0, 0, 0, 0, 'chinese');
TemporalHelpers.assertDuration(g.until(h, { largestUnit: 'years' }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0, "Chinese calendar, 1 year");

