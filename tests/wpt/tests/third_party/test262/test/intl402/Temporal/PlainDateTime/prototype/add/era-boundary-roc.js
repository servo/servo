// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: >
  Adding years works correctly across era boundaries in calendars with eras
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration1 = new Temporal.Duration(1);
const duration1n = new Temporal.Duration(-1);
const calendar = "roc";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDateTime.from({ era: "broc", eraYear: 2, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1.add(duration1),
  0, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 2 BROC lands in 1 BROC (counts backwards)",
  "broc", 1
);

const date2 = Temporal.PlainDateTime.from({ era: "broc", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date2.add(duration1),
  1, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 1 BROC lands in 1 ROC (no year zero)",
  "roc", 1
);

const date3 = Temporal.PlainDateTime.from({ era: "roc", eraYear: 1, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date3.add(duration1),
  2, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to 1 ROC lands in 2 ROC",
  "roc", 2
);

const date4 = Temporal.PlainDateTime.from({ era: "broc", eraYear: 5, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date4.add(new Temporal.Duration(10)),
  6, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Adding 10 years to 5 BROC lands in 6 ROC (no year zero)",
  "roc", 6
);

const date5 = Temporal.PlainDateTime.from({ era: "roc", eraYear: 5, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date5.add(duration1n),
  4, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from ROC 5 lands in ROC 4",
  "roc", 4
);

TemporalHelpers.assertPlainDateTime(
  date3.add(duration1n),
  0, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from ROC 1 lands in BROC 1",
  "broc", 1
);

const date6 = Temporal.PlainDateTime.from({ era: "roc", eraYear: 10, monthCode: "M03", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date6.add(new Temporal.Duration(-15)),
  -5, 3, "M03", 1, 12, 34, 0, 0, 0, 0, "Subtracting 15 years from ROC 10 lands in BROC 6",
  "broc", 6
);
