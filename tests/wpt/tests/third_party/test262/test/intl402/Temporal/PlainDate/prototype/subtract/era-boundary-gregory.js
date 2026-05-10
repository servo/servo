// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Adding years works correctly across era boundaries in gregory calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration1 = new Temporal.Duration(-1);
const duration1n = new Temporal.Duration(1);
const calendar = "gregory";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDate.from({ era: "bce", eraYear: 2, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date1.subtract(duration1),
  0, 6, "M06", 15, "Adding 1 year to 2 BCE lands in 1 BCE (counts backwards)",
  "bce", 1
);

const date2 = Temporal.PlainDate.from({ era: "bce", eraYear: 1, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date2.subtract(duration1),
  1, 6, "M06", 15, "Adding 1 year to 1 BCE lands in 1 CE (no year zero)",
  "ce", 1
);

const date3 = Temporal.PlainDate.from({ era: "ce", eraYear: 1, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date3.subtract(duration1),
  2, 6, "M06", 15, "Adding 1 year to 1 CE lands in 2 CE",
  "ce", 2
);

const date4 = Temporal.PlainDate.from({ era: "bce", eraYear: 5, monthCode: "M03", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date4.subtract(new Temporal.Duration(-10)),
  6, 3, "M03", 1, "Adding 10 years to 5 BCE lands in 6 CE (no year zero)",
  "ce", 6
);

const date5 = Temporal.PlainDate.from({ era: "ce", eraYear: 2, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date5.subtract(duration1n),
  1, 6, "M06", 15, "Subtracting 1 year from 2 CE lands in 1 CE",
  "ce", 1
);

TemporalHelpers.assertPlainDate(
  date3.subtract(duration1n),
  0, 6, "M06", 15, "Subtracting 1 year from 1 CE lands in 1 BCE",
  "bce", 1
);

TemporalHelpers.assertPlainDate(
  date2.subtract(duration1n),
  -1, 6, "M06", 15, "Subtracting 1 year from 1 BCE lands in 2 BCE",
  "bce", 2
);

const date6 = Temporal.PlainDate.from({ era: "ce", eraYear: 5, monthCode: "M03", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date6.subtract(new Temporal.Duration(10)),
  -5, 3, "M03", 1, "Subtracting 10 years from 5 CE lands in 6 BCE",
  "bce", 6
);
