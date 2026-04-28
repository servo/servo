// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Adding years works correctly across era boundaries in gregory calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration1 = new Temporal.Duration(1);
const duration1n = new Temporal.Duration(-1);
const calendar = "gregory";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDateTime.from({ era: "bce", eraYear: 2, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1.add(duration1),
  0, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 2 BCE lands in 1 BCE (counts backwards)",
  "bce", 1
);

const date2 = Temporal.PlainDateTime.from({ era: "bce", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date2.add(duration1),
  1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 1 BCE lands in 1 CE (no year zero)",
  "ce", 1
);

const date3 = Temporal.PlainDateTime.from({ era: "ce", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date3.add(duration1),
  2, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 1 CE lands in 2 CE",
  "ce", 2
);

const date4 = Temporal.PlainDateTime.from({ era: "bce", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date4.add(new Temporal.Duration(10)),
  6, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 10 years to 5 BCE lands in 6 CE (no year zero)",
  "ce", 6
);

const date5 = Temporal.PlainDateTime.from({ era: "ce", eraYear: 2, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date5.add(duration1n),
  1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from 2 CE lands in 1 CE",
  "ce", 1
);

TemporalHelpers.assertPlainDateTime(
  date3.add(duration1n),
  0, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from 1 CE lands in 1 BCE",
  "bce", 1
);

TemporalHelpers.assertPlainDateTime(
  date2.add(duration1n),
  -1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from 1 BCE lands in 2 BCE",
  "bce", 2
);

const date6 = Temporal.PlainDateTime.from({ era: "ce", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date6.add(new Temporal.Duration(-10)),
  -5, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Subtracting 10 years from 5 CE lands in 6 BCE",
  "bce", 6
);
