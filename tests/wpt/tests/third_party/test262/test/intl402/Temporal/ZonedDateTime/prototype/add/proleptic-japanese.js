// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: >
  Check that Japanese calendar is implemented as proleptic
  (japanese calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "japanese";

const days3 = new Temporal.Duration(0, 0, 0, 3);
const days3n = new Temporal.Duration(0, 0, 0, -3);
const weeks1 = new Temporal.Duration(0, 0, 1);
const weeks1n = new Temporal.Duration(0, 0, -1);

const date15821004 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 4, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date15821015 = Temporal.ZonedDateTime.from({ year: 1582, monthCode: "M10", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
TemporalHelpers.assertPlainDateTime(
  date15821004.add(days3).toPlainDateTime(),
  1582, 10, "M10", 7, 12, 34, 0, 0, 0, 0, "add 3 days to 1582-10-04",
  "ce", 1582);
TemporalHelpers.assertPlainDateTime(
  date15821015.add(days3n).toPlainDateTime(),
  1582, 10, "M10", 12, 12, 34, 0, 0, 0, 0, "subtract 3 days from 1582-10-15",
  "ce", 1582);
TemporalHelpers.assertPlainDateTime(
  date15821004.add(weeks1).toPlainDateTime(),
  1582, 10, "M10", 11, 12, 34, 0, 0, 0, 0, "add 1 week to 1582-10-04",
  "ce", 1582);
TemporalHelpers.assertPlainDateTime(
  date15821015.add(weeks1n).toPlainDateTime(),
  1582, 10, "M10", 8, 12, 34, 0, 0, 0, 0, "subtract 1 week from 1582-10-15",
  "ce", 1582);
