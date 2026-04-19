// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Dates in same year or years before era starts should resolve to previous era
  (Japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const reiwa1BeforeStart = Temporal.ZonedDateTime.from({ era: "reiwa", eraYear: 1, monthCode: "M04", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  reiwa1BeforeStart.toPlainDateTime(),
  2019, 4, "M04", 30, 12, 34, 0, 0, 0, 0, "Reiwa 1 resolves to Heisei 31 before era start date",
  "heisei", 31);

const reiwa0 = Temporal.ZonedDateTime.from({ era: "reiwa", eraYear: 0, monthCode: "M05", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  reiwa0.toPlainDateTime(),
  2018, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Reiwa 0 resolves to Heisei 30",
  "heisei", 30);

const reiwa1n = Temporal.ZonedDateTime.from({ era: "reiwa", eraYear: -1, monthCode: "M05", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  reiwa1n.toPlainDateTime(),
  2017, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Reiwa -1 resolves to Heisei 29",
  "heisei", 29);

const heisei31AfterStart = Temporal.ZonedDateTime.from({ era: "heisei", eraYear: 31, monthCode: "M05", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  heisei31AfterStart.toPlainDateTime(),
  2019, 5, "M05", 1, 12, 34, 0, 0, 0, 0, "Heisei 31 resolves to Reiwa 1 after era start date",
  "reiwa", 1);

const heisei1BeforeStart = Temporal.ZonedDateTime.from({ era: "heisei", eraYear: 1, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  heisei1BeforeStart.toPlainDateTime(),
  1989, 1, "M01", 7, 12, 34, 0, 0, 0, 0, "Heisei 1 resolves to Showa 64 before era start date",
  "showa", 64);

const heisei0 = Temporal.ZonedDateTime.from({ era: "heisei", eraYear: 0, monthCode: "M01", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  heisei0.toPlainDateTime(),
  1988, 1, "M01", 8, 12, 34, 0, 0, 0, 0, "Heisei 0 resolves to Showa 63",
  "showa", 63);

const heisei1n = Temporal.ZonedDateTime.from({ era: "heisei", eraYear: -1, monthCode: "M01", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  heisei1n.toPlainDateTime(),
  1987, 1, "M01", 8, 12, 34, 0, 0, 0, 0, "Heisei -1 resolves to Showa 62",
  "showa", 62);

const showa64AfterStart = Temporal.ZonedDateTime.from({ era: "showa", eraYear: 64, monthCode: "M01", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  showa64AfterStart.toPlainDateTime(),
  1989, 1, "M01", 8, 12, 34, 0, 0, 0, 0, "Showa 64 resolves to Heisei 1 after era start date",
  "heisei", 1);

const showa1BeforeStart = Temporal.ZonedDateTime.from({ era: "showa", eraYear: 1, monthCode: "M12", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  showa1BeforeStart.toPlainDateTime(),
  1926, 12, "M12", 24, 12, 34, 0, 0, 0, 0, "Showa 1 resolves to Taisho 15 before era start date",
  "taisho", 15);

const showa0 = Temporal.ZonedDateTime.from({ era: "showa", eraYear: 0, monthCode: "M12", day: 25, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  showa0.toPlainDateTime(),
  1925, 12, "M12", 25, 12, 34, 0, 0, 0, 0, "Showa 0 resolves to Taisho 14",
  "taisho", 14);

const showa1n = Temporal.ZonedDateTime.from({ era: "showa", eraYear: -1, monthCode: "M12", day: 25, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  showa1n.toPlainDateTime(),
  1924, 12, "M12", 25, 12, 34, 0, 0, 0, 0, "Showa -1 resolves to Taisho 13",
  "taisho", 13);

const taisho15AfterStart = Temporal.ZonedDateTime.from({ era: "taisho", eraYear: 15, monthCode: "M12", day: 25, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  taisho15AfterStart.toPlainDateTime(),
  1926, 12, "M12", 25, 12, 34, 0, 0, 0, 0, "Taisho 15 resolves to Showa 1 after era start date",
  "showa", 1);

const taisho1BeforeStart = Temporal.ZonedDateTime.from({ era: "taisho", eraYear: 1, monthCode: "M07", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  taisho1BeforeStart.toPlainDateTime(),
  1912, 7, "M07", 29, 12, 34, 0, 0, 0, 0, "Taisho 1 resolves to Meiji 45 before era start date",
  "meiji", 45);

const taisho0 = Temporal.ZonedDateTime.from({ era: "taisho", eraYear: 0, monthCode: "M07", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  taisho0.toPlainDateTime(),
  1911, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "Taisho 0 resolves to Meiji 44",
  "meiji", 44);

const taisho1n = Temporal.ZonedDateTime.from({ era: "taisho", eraYear: -1, monthCode: "M07", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  taisho1n.toPlainDateTime(),
  1910, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "Taisho -1 resolves to Meiji 43",
  "meiji", 43);

const meiji45AfterStart = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 45, monthCode: "M07", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji45AfterStart.toPlainDateTime(),
  1912, 7, "M07", 30, 12, 34, 0, 0, 0, 0, "Meiji 45 resolves to Taisho 1 after era start date",
  "taisho", 1);

const meiji1BeforeStart = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 1, monthCode: "M10", day: 22, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji1BeforeStart.toPlainDateTime(),
  1868, 10, "M10", 22, 12, 34, 0, 0, 0, 0, "Meiji 1 resolves to CE 1868 before era start date",
  "ce", 1868);

const meiji1AfterStart = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 1, monthCode: "M10", day: 23, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji1AfterStart.toPlainDateTime(),
  1868, 10, "M10", 23, 12, 34, 0, 0, 0, 0, "Meiji 1 still resolves to CE 1868 after era start date",
  "ce", 1868);

const meiji5 = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 5, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji5.toPlainDateTime(),
  1872, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "Meiji 5 resolves to CE 1872",
  "ce", 1872);

const ce1873 = Temporal.ZonedDateTime.from({ era: "ce", eraYear: 1873, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  ce1873.toPlainDateTime(),
  1873, 1, "M01", 1, 12, 34, 0, 0, 0, 0, "CE 1873 resolves to Meiji 6",
  "meiji", 6);

const meiji0 = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 0, monthCode: "M10", day: 23, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji0.toPlainDateTime(),
  1867, 10, "M10", 23, 12, 34, 0, 0, 0, 0, "Meiji 0 resolves to CE 1867",
  "ce", 1867);

const meiji1n = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: -1, monthCode: "M10", day: 23, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji1n.toPlainDateTime(),
  1866, 10, "M10", 23, 12, 34, 0, 0, 0, 0, "Meiji -1 resolves to CE 1866",
  "ce", 1866);

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

// Years far after the end of an era

const heisei100 = Temporal.ZonedDateTime.from({ era: "heisei", eraYear: 100, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  heisei100.toPlainDateTime(),
  2088, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "Heisei 100 resolves to Reiwa 70",
  "reiwa", 70);

const showa100 = Temporal.ZonedDateTime.from({ era: "showa", eraYear: 100, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  showa100.toPlainDateTime(),
  2025, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "Showa 100 resolves to Reiwa 7",
  "reiwa", 7);

const taisho100 = Temporal.ZonedDateTime.from({ era: "taisho", eraYear: 100, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  taisho100.toPlainDateTime(),
  2011, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "Taisho 100 resolves to Heisei 23",
  "heisei", 23);

const meiji100 = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 100, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  meiji100.toPlainDateTime(),
  1967, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "Meiji 100 resolves to Showa 42",
  "showa", 42);

const ce2000 = Temporal.ZonedDateTime.from({ era: "ce", eraYear: 2000, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(
  ce2000.toPlainDateTime(),
  2000, 12, "M12", 31, 12, 34, 0, 0, 0, 0, "CE 2000 resolves to Heisei 12",
  "heisei", 12);
