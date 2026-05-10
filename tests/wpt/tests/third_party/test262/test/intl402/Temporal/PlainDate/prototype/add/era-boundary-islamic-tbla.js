// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Adding years works correctly across era boundaries in islamic-tbla calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration1 = new Temporal.Duration(1);
const duration1n = new Temporal.Duration(-1);
const calendar = "islamic-tbla";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDate.from({ era: "bh", eraYear: 2, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date1.add(duration1),
  0, 6, "M06", 15, "Adding 1 year to 2 BH lands in 1 BH (counts backwards)",
  "bh", 1
);

const date2 = Temporal.PlainDate.from({ era: "bh", eraYear: 1, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date2.add(duration1),
  1, 6, "M06", 15, "Adding 1 year to 1 BH lands in 1 AH (no year zero)",
  "ah", 1
);

const date3 = Temporal.PlainDate.from({ era: "ah", eraYear: 1, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date3.add(duration1),
  2, 6, "M06", 15, "Adding 1 year to 1 AH lands in 2 AH",
  "ah", 2
);

const date4 = Temporal.PlainDate.from({ era: "bh", eraYear: 5, monthCode: "M03", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date4.add(new Temporal.Duration(10)),
  6, 3, "M03", 1, "Adding 10 years to 5 BH lands in 6 AH (no year zero)",
  "ah", 6
);

const date5 = Temporal.PlainDate.from({ era: "ah", eraYear: 2, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date5.add(duration1n),
  1, 6, "M06", 15, "Subtracting 1 year from 2 AH lands in 1 AH",
  "ah", 1
);

TemporalHelpers.assertPlainDate(
  date3.add(duration1n),
  0, 6, "M06", 15, "Subtracting 1 year from 1 AH lands in 1 BH",
  "bh", 1
);

TemporalHelpers.assertPlainDate(
  date2.add(duration1n),
  -1, 6, "M06", 15, "Subtracting 1 year from 1 BH lands in 2 BH",
  "bh", 2
);

const date6 = Temporal.PlainDate.from({ era: "ah", eraYear: 5, monthCode: "M03", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date6.add(new Temporal.Duration(-10)),
  -5, 3, "M03", 1, "Subtracting 10 years from 5 AH lands in 6 BH",
  "bh", 6
);
