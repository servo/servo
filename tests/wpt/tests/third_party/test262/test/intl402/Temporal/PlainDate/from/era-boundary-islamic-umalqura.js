// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Non-positive era years are remapped in islamic-umalqura calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-umalqura";
const options = { overflow: "reject" };

const ah0 = Temporal.PlainDate.from({ era: "ah", eraYear: 0, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  ah0,
  0, 1, "M01", 1, "AH 0 resolves to BH 1",
  "bh", 1);

const ah1n = Temporal.PlainDate.from({ era: "ah", eraYear: -1, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  ah1n,
  -1, 1, "M01", 1, "AH -1 resolves to BH 2",
  "bh", 2);

const bh0 = Temporal.PlainDate.from({ era: "bh", eraYear: 0, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  bh0,
  1, 1, "M01", 1, "BH 0 resolves to AH 1",
  "ah", 1);

const bh1n = Temporal.PlainDate.from({ era: "bh", eraYear: -1, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  bh1n,
  2, 1, "M01", 1, "BH -1 resolves to AH 2",
  "ah", 2);
