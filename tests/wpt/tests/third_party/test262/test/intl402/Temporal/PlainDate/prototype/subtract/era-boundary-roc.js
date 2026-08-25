// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: >
  Adding years works correctly across era boundaries in calendars with eras
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration1 = new Temporal.Duration(-1);
const duration1n = new Temporal.Duration(1);
const calendar = "roc";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDate.from({ era: "broc", eraYear: 2, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date1.subtract(duration1),
  0, 6, "M06", 15, "Adding 1 year to 2 BROC lands in 1 BROC (counts backwards)",
  "broc", 1
);

const date2 = Temporal.PlainDate.from({ era: "broc", eraYear: 1, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date2.subtract(duration1),
  1, 6, "M06", 15, "Adding 1 year to 1 BROC lands in 1 ROC (no year zero)",
  "roc", 1
);

const date3 = Temporal.PlainDate.from({ era: "roc", eraYear: 1, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date3.subtract(duration1),
  2, 6, "M06", 15, "Adding 1 year to 1 ROC lands in 2 ROC",
  "roc", 2
);

const date4 = Temporal.PlainDate.from({ era: "broc", eraYear: 5, monthCode: "M03", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date4.subtract(new Temporal.Duration(-10)),
  6, 3, "M03", 1, "Adding 10 years to 5 BROC lands in 6 ROC (no year zero)",
  "roc", 6
);

const date5 = Temporal.PlainDate.from({ era: "roc", eraYear: 5, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date5.subtract(duration1n),
  4, 6, "M06", 15, "Subtracting 1 year from ROC 5 lands in ROC 4",
  "roc", 4
);

TemporalHelpers.assertPlainDate(
  date3.subtract(duration1n),
  0, 6, "M06", 15, "Subtracting 1 year from ROC 1 lands in BROC 1",
  "broc", 1
);

const date6 = Temporal.PlainDate.from({ era: "roc", eraYear: 10, monthCode: "M03", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date6.subtract(new Temporal.Duration(15)),
  -5, 3, "M03", 1, "Subtracting 15 years from ROC 10 lands in BROC 6",
  "broc", 6
);
