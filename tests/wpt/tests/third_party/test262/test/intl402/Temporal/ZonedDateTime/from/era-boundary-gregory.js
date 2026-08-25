// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Non-positive era years are remapped in gregory calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

const ce0 = Temporal.ZonedDateTime.from({ era: "ce", eraYear: 0, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  ce0.toPlainDateTime(),
  0, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "CE 0 resolves to BCE 1",
  "bce", 1);

const ce1n = Temporal.ZonedDateTime.from({ era: "ce", eraYear: -1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  ce1n.toPlainDateTime(),
  -1, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "CE -1 resolves to BCE 2",
  "bce", 2);

const bce0 = Temporal.ZonedDateTime.from({ era: "bce", eraYear: 0, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  bce0.toPlainDateTime(),
  1, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "BCE 0 resolves to CE 1",
  "ce", 1);

const bce1n = Temporal.ZonedDateTime.from({ era: "bce", eraYear: -1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  bce1n.toPlainDateTime(),
  2, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "BCE -1 resolves to CE 2",
  "ce", 2);
