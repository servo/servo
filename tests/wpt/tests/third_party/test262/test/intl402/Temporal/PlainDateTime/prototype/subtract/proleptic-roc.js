// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: >
  Check that roc calendar is implemented as proleptic
  (roc calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "roc";

const days3 = new Temporal.Duration(0, 0, 0, -3);
const days3n = new Temporal.Duration(0, 0, 0, 3);
const weeks1 = new Temporal.Duration(0, 0, -1);
const weeks1n = new Temporal.Duration(0, 0, 1);

const date329n1004 = Temporal.PlainDateTime.from({ year: -329, monthCode: "M10", day: 4, hour: 12, minute: 34, calendar });
const date329n1015 = Temporal.PlainDateTime.from({ year: -329, monthCode: "M10", day: 15, hour: 12, minute: 34, calendar });
TemporalHelpers.assertPlainDateTime(
  date329n1004.subtract(days3),
  -329, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 3 days to -329-10-04",
  "broc", 330);
TemporalHelpers.assertPlainDateTime(
  date329n1015.subtract(days3n),
  -329, 10, "M10", 12, 12, 34, 0, 0, 0, 0, "subtract 3 days from -329-10-15",
  "broc", 330);
TemporalHelpers.assertPlainDateTime(
  date329n1004.subtract(weeks1),
  -329, 10, "M10", 11, 12, 34, 0, 0, 0, 0, "add 1 week to -329-10-04",
  "broc", 330);
TemporalHelpers.assertPlainDateTime(
  date329n1015.subtract(weeks1n),
  -329, 10, "M10", 8, 12, 34, 0, 0, 0, 0, "subtract 1 week from -329-10-15",
  "broc", 330);
