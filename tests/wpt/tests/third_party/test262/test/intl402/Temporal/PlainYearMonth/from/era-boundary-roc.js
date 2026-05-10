// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Non-positive era years are remapped in roc calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";
const options = { overflow: "reject" };

const roc0 = Temporal.PlainYearMonth.from({ era: "roc", eraYear: 0, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  roc0,
  0, 1, "M01", "ROC 0 resolves to BROC 1",
  "broc", 1);

const roc1n = Temporal.PlainYearMonth.from({ era: "roc", eraYear: -1, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  roc1n,
  -1, 1, "M01", "ROC -1 resolves to BROC 2",
  "broc", 2);

const broc0 = Temporal.PlainYearMonth.from({ era: "broc", eraYear: 0, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  broc0,
  1, 1, "M01", "BROC 0 resolves to ROC 1",
  "roc", 1);

const broc1n = Temporal.PlainYearMonth.from({ era: "broc", eraYear: -1, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  broc1n,
  2, 1, "M01", "BROC -1 resolves to ROC 2",
  "roc", 2);
