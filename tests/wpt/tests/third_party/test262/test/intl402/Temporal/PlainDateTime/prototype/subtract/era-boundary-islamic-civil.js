// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Adding years works correctly across era boundaries in islamic-civil calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration1 = new Temporal.Duration(-1);
const duration1n = new Temporal.Duration(1);
const calendar = "islamic-civil";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDateTime.from({ era: "bh", eraYear: 2, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1.subtract(duration1),
  0, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 2 BH lands in 1 BH (counts backwards)",
  "bh", 1
);

const date2 = Temporal.PlainDateTime.from({ era: "bh", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date2.subtract(duration1),
  1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 1 BH lands in 1 AH (no year zero)",
  "ah", 1
);

const date3 = Temporal.PlainDateTime.from({ era: "ah", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date3.subtract(duration1),
  2, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 1 AH lands in 2 AH",
  "ah", 2
);

const date4 = Temporal.PlainDateTime.from({ era: "bh", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date4.subtract(new Temporal.Duration(-10)),
  6, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 10 years to 5 BH lands in 6 AH (no year zero)",
  "ah", 6
);

const date5 = Temporal.PlainDateTime.from({ era: "ah", eraYear: 2, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date5.subtract(duration1n),
  1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from 2 AH lands in 1 AH",
  "ah", 1
);

TemporalHelpers.assertPlainDateTime(
  date3.subtract(duration1n),
  0, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from 1 AH lands in 1 BH",
  "bh", 1
);

TemporalHelpers.assertPlainDateTime(
  date2.subtract(duration1n),
  -1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from 1 BH lands in 2 BH",
  "bh", 2
);

const date6 = Temporal.PlainDateTime.from({ era: "ah", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date6.subtract(new Temporal.Duration(10)),
  -5, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Subtracting 10 years from 5 AH lands in 6 BH",
  "bh", 6
);
