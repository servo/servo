// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Non-positive era years are remapped in islamic-tbla calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-tbla";
const options = { overflow: "reject" };

const ah0 = Temporal.ZonedDateTime.from({ era: "ah", eraYear: 0, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  ah0.toPlainDateTime(),
  0, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "AH 0 resolves to BH 1",
  "bh", 1);

const ah1n = Temporal.ZonedDateTime.from({ era: "ah", eraYear: -1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  ah1n.toPlainDateTime(),
  -1, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "AH -1 resolves to BH 2",
  "bh", 2);

const bh0 = Temporal.ZonedDateTime.from({ era: "bh", eraYear: 0, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  bh0.toPlainDateTime(),
  1, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "BH 0 resolves to AH 1",
  "ah", 1);

const bh1n = Temporal.ZonedDateTime.from({ era: "bh", eraYear: -1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  bh1n.toPlainDateTime(),
  2, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "BH -1 resolves to AH 2",
  "ah", 2);
