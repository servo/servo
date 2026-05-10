// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Non-positive era years are remapped in islamic-civil calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-civil";
const options = { overflow: "reject" };

const ah0 = Temporal.PlainYearMonth.from({ era: "ah", eraYear: 0, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  ah0,
  0, 1, "M01", "AH 0 resolves to BH 1",
  "bh", 1, null);

const ah1n = Temporal.PlainYearMonth.from({ era: "ah", eraYear: -1, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  ah1n,
  -1, 1, "M01", "AH -1 resolves to BH 2",
  "bh", 2, null);

const bh0 = Temporal.PlainYearMonth.from({ era: "bh", eraYear: 0, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  bh0,
  1, 1, "M01", "BH 0 resolves to AH 1",
  "ah", 1, null);

const bh1n = Temporal.PlainYearMonth.from({ era: "bh", eraYear: -1, monthCode: "M01", calendar }, options);
TemporalHelpers.assertPlainYearMonth(
  bh1n,
  2, 1, "M01", "BH -1 resolves to AH 2",
  "ah", 2, null);
