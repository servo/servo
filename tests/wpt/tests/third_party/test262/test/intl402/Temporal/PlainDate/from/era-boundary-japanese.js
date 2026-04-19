// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Dates in same year or years before era starts should resolve to previous era
  (Japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const reiwa1BeforeStart = Temporal.PlainDate.from({ era: "reiwa", eraYear: 1, monthCode: "M04", day: 30, calendar }, options);
TemporalHelpers.assertPlainDate(
  reiwa1BeforeStart,
  2019, 4, "M04", 30, "Reiwa 1 resolves to Heisei 31 before era start date",
  "heisei", 31);

const reiwa0 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 0, monthCode: "M05", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  reiwa0,
  2018, 5, "M05", 1, "Reiwa 0 resolves to Heisei 30",
  "heisei", 30);

const reiwa1n = Temporal.PlainDate.from({ era: "reiwa", eraYear: -1, monthCode: "M05", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  reiwa1n,
  2017, 5, "M05", 1, "Reiwa -1 resolves to Heisei 29",
  "heisei", 29);

const heisei31AfterStart = Temporal.PlainDate.from({ era: "heisei", eraYear: 31, monthCode: "M05", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  heisei31AfterStart,
  2019, 5, "M05", 1, "Heisei 31 resolves to Reiwa 1 after era start date",
  "reiwa", 1);

const heisei1BeforeStart = Temporal.PlainDate.from({ era: "heisei", eraYear: 1, monthCode: "M01", day: 7, calendar }, options);
TemporalHelpers.assertPlainDate(
  heisei1BeforeStart,
  1989, 1, "M01", 7, "Heisei 1 resolves to Showa 64 before era start date",
  "showa", 64);

const heisei0 = Temporal.PlainDate.from({ era: "heisei", eraYear: 0, monthCode: "M01", day: 8, calendar }, options);
TemporalHelpers.assertPlainDate(
  heisei0,
  1988, 1, "M01", 8, "Heisei 0 resolves to Showa 63",
  "showa", 63);

const heisei1n = Temporal.PlainDate.from({ era: "heisei", eraYear: -1, monthCode: "M01", day: 8, calendar }, options);
TemporalHelpers.assertPlainDate(
  heisei1n,
  1987, 1, "M01", 8, "Heisei -1 resolves to Showa 62",
  "showa", 62);

const showa64AfterStart = Temporal.PlainDate.from({ era: "showa", eraYear: 64, monthCode: "M01", day: 8, calendar }, options);
TemporalHelpers.assertPlainDate(
  showa64AfterStart,
  1989, 1, "M01", 8, "Showa 64 resolves to Heisei 1 after era start date",
  "heisei", 1);

const showa1BeforeStart = Temporal.PlainDate.from({ era: "showa", eraYear: 1, monthCode: "M12", day: 24, calendar }, options);
TemporalHelpers.assertPlainDate(
  showa1BeforeStart,
  1926, 12, "M12", 24, "Showa 1 resolves to Taisho 15 before era start date",
  "taisho", 15);

const showa0 = Temporal.PlainDate.from({ era: "showa", eraYear: 0, monthCode: "M12", day: 25, calendar }, options);
TemporalHelpers.assertPlainDate(
  showa0,
  1925, 12, "M12", 25, "Showa 0 resolves to Taisho 14",
  "taisho", 14);

const showa1n = Temporal.PlainDate.from({ era: "showa", eraYear: -1, monthCode: "M12", day: 25, calendar }, options);
TemporalHelpers.assertPlainDate(
  showa1n,
  1924, 12, "M12", 25, "Showa -1 resolves to Taisho 13",
  "taisho", 13);

const taisho15AfterStart = Temporal.PlainDate.from({ era: "taisho", eraYear: 15, monthCode: "M12", day: 25, calendar }, options);
TemporalHelpers.assertPlainDate(
  taisho15AfterStart,
  1926, 12, "M12", 25, "Taisho 15 resolves to Showa 1 after era start date",
  "showa", 1);

const taisho1BeforeStart = Temporal.PlainDate.from({ era: "taisho", eraYear: 1, monthCode: "M07", day: 29, calendar }, options);
TemporalHelpers.assertPlainDate(
  taisho1BeforeStart,
  1912, 7, "M07", 29, "Taisho 1 resolves to Meiji 45 before era start date",
  "meiji", 45);

const taisho0 = Temporal.PlainDate.from({ era: "taisho", eraYear: 0, monthCode: "M07", day: 30, calendar }, options);
TemporalHelpers.assertPlainDate(
  taisho0,
  1911, 7, "M07", 30, "Taisho 0 resolves to Meiji 44",
  "meiji", 44);

const taisho1n = Temporal.PlainDate.from({ era: "taisho", eraYear: -1, monthCode: "M07", day: 30, calendar }, options);
TemporalHelpers.assertPlainDate(
  taisho1n,
  1910, 7, "M07", 30, "Taisho -1 resolves to Meiji 43",
  "meiji", 43);

const meiji45AfterStart = Temporal.PlainDate.from({ era: "meiji", eraYear: 45, monthCode: "M07", day: 30, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji45AfterStart,
  1912, 7, "M07", 30, "Meiji 45 resolves to Taisho 1 after era start date",
  "taisho", 1);

const meiji1BeforeStart = Temporal.PlainDate.from({ era: "meiji", eraYear: 1, monthCode: "M10", day: 22, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji1BeforeStart,
  1868, 10, "M10", 22, "Meiji 1 resolves to CE 1868 before era start date",
  "ce", 1868);

const meiji1AfterStart = Temporal.PlainDate.from({ era: "meiji", eraYear: 1, monthCode: "M10", day: 23, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji1AfterStart,
  1868, 10, "M10", 23, "Meiji 1 still resolves to CE 1868 after era start date",
  "ce", 1868);

const meiji5 = Temporal.PlainDate.from({ era: "meiji", eraYear: 5, monthCode: "M12", day: 31, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji5,
  1872, 12, "M12", 31, "Meiji 5 resolves to CE 1872",
  "ce", 1872);

const ce1873 = Temporal.PlainDate.from({ era: "ce", eraYear: 1873, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  ce1873,
  1873, 1, "M01", 1, "CE 1873 resolves to Meiji 6",
  "meiji", 6);

const meiji0 = Temporal.PlainDate.from({ era: "meiji", eraYear: 0, monthCode: "M10", day: 23, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji0,
  1867, 10, "M10", 23, "Meiji 0 resolves to CE 1867",
  "ce", 1867);

const meiji1n = Temporal.PlainDate.from({ era: "meiji", eraYear: -1, monthCode: "M10", day: 23, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji1n,
  1866, 10, "M10", 23, "Meiji -1 resolves to CE 1866",
  "ce", 1866);

const ce0 = Temporal.PlainDate.from({ era: "ce", eraYear: 0, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  ce0,
  0, 1, "M01", 1, "CE 0 resolves to BCE 1",
  "bce", 1);

const ce1n = Temporal.PlainDate.from({ era: "ce", eraYear: -1, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  ce1n,
  -1, 1, "M01", 1, "CE -1 resolves to BCE 2",
  "bce", 2);

const bce0 = Temporal.PlainDate.from({ era: "bce", eraYear: 0, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  bce0,
  1, 1, "M01", 1, "BCE 0 resolves to CE 1",
  "ce", 1);

const bce1n = Temporal.PlainDate.from({ era: "bce", eraYear: -1, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  bce1n,
  2, 1, "M01", 1, "BCE -1 resolves to CE 2",
  "ce", 2);

// Years far after the end of an era

const heisei100 = Temporal.PlainDate.from({ era: "heisei", eraYear: 100, monthCode: "M12", day: 31, calendar }, options);
TemporalHelpers.assertPlainDate(
  heisei100,
  2088, 12, "M12", 31, "Heisei 100 resolves to Reiwa 70",
  "reiwa", 70);

const showa100 = Temporal.PlainDate.from({ era: "showa", eraYear: 100, monthCode: "M12", day: 31, calendar }, options);
TemporalHelpers.assertPlainDate(
  showa100,
  2025, 12, "M12", 31, "Showa 100 resolves to Reiwa 7",
  "reiwa", 7);

const taisho100 = Temporal.PlainDate.from({ era: "taisho", eraYear: 100, monthCode: "M12", day: 31, calendar }, options);
TemporalHelpers.assertPlainDate(
  taisho100,
  2011, 12, "M12", 31, "Taisho 100 resolves to Heisei 23",
  "heisei", 23);

const meiji100 = Temporal.PlainDate.from({ era: "meiji", eraYear: 100, monthCode: "M12", day: 31, calendar }, options);
TemporalHelpers.assertPlainDate(
  meiji100,
  1967, 12, "M12", 31, "Meiji 100 resolves to Showa 42",
  "showa", 42);

const ce2000 = Temporal.PlainDate.from({ era: "ce", eraYear: 2000, monthCode: "M12", day: 31, calendar }, options);
TemporalHelpers.assertPlainDate(
  ce2000,
  2000, 12, "M12", 31, "CE 2000 resolves to Heisei 12",
  "heisei", 12);
