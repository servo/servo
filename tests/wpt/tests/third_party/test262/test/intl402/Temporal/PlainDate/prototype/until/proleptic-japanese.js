// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: >
  Check that Japanese calendar is implemented as proleptic
  (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";

const date15821004 = Temporal.PlainDate.from({ year: 1582, monthCode: "M10", day: 4, calendar });
const date15821007 = Temporal.PlainDate.from({ year: 1582, monthCode: "M10", day: 7, calendar });
const date15821011 = Temporal.PlainDate.from({ year: 1582, monthCode: "M10", day: 11, calendar });
const date15821012 = Temporal.PlainDate.from({ year: 1582, monthCode: "M10", day: 12, calendar });
const date15821015 = Temporal.PlainDate.from({ year: 1582, monthCode: "M10", day: 15, calendar });
TemporalHelpers.assertDuration(
  date15821004.until(date15821007, { largestUnit: "days" }),
  0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
  "1582-10-04 and 1582-10-07");
TemporalHelpers.assertDuration(
  date15821015.until(date15821012, { largestUnit: "days" }),
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "1582-10-15 and 1582-10-12");
TemporalHelpers.assertDuration(
  date15821004.until(date15821011, { largestUnit: "weeks" }),
  0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
  "1582-10-04 and 1582-10-11")
TemporalHelpers.assertDuration(
  date15821011.until(date15821004, { largestUnit: "weeks" }),
  0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
  "1582-10-11 and 1582-10-04")
