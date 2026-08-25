// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Non-positive era years are remapped in roc calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const roc0 = Temporal.ZonedDateTime.from({ era: "roc", eraYear: 0, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  roc0.toPlainDateTime(),
  0, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "ROC 0 resolves to BROC 1",
  "broc", 1);

const roc1n = Temporal.ZonedDateTime.from({ era: "roc", eraYear: -1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  roc1n.toPlainDateTime(),
  -1, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "ROC -1 resolves to BROC 2",
  "broc", 2);

const broc0 = Temporal.ZonedDateTime.from({ era: "broc", eraYear: 0, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  broc0.toPlainDateTime(),
  1, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "BROC 0 resolves to ROC 1",
  "roc", 1);

const broc1n = Temporal.ZonedDateTime.from({ era: "broc", eraYear: -1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  broc1n.toPlainDateTime(),
  2, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "BROC -1 resolves to ROC 2",
  "roc", 2);
