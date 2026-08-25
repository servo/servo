// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: >
  Check that ROC calendar is implemented as proleptic
  (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";

const date329n1004 = Temporal.PlainDate.from({ year: -329, monthCode: "M10", day: 4, calendar });
const date329n1007 = Temporal.PlainDate.from({ year: -329, monthCode: "M10", day: 7, calendar });
const date329n1011 = Temporal.PlainDate.from({ year: -329, monthCode: "M10", day: 11, calendar });
const date329n1012 = Temporal.PlainDate.from({ year: -329, monthCode: "M10", day: 12, calendar });
const date329n1015 = Temporal.PlainDate.from({ year: -329, monthCode: "M10", day: 15, calendar });
TemporalHelpers.assertDuration(
  date329n1004.since(date329n1007, { largestUnit: "days" }),
  0, 0, 0, -3, 0, 0, 0, 0, 0, 0,
  "-329-10-04 and -329-10-07");
TemporalHelpers.assertDuration(
  date329n1015.since(date329n1012, { largestUnit: "days" }),
  0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
  "-329-10-15 and -329-10-12");
TemporalHelpers.assertDuration(
  date329n1004.since(date329n1011, { largestUnit: "weeks" }),
  0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
  "-329-10-04 and -329-10-11")
TemporalHelpers.assertDuration(
  date329n1011.since(date329n1004, { largestUnit: "weeks" }),
  0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
  "-329-10-11 and -329-10-04")
