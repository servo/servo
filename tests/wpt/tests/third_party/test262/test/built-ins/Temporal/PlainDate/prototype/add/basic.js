// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Basic tests
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = Temporal.PlainDate.from("1976-11-18");
TemporalHelpers.assertPlainDate(date.add({ years: 43 }),
  2019, 11, "M11", 18);
TemporalHelpers.assertPlainDate(date.add({ months: 3 }),
  1977, 2, "M02", 18);
TemporalHelpers.assertPlainDate(date.add({ days: 20 }),
  1976, 12, "M12", 8);
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from("2019-01-31").add({ months: 1 }),
  2019, 2, "M02", 28);
TemporalHelpers.assertPlainDate(date.add(Temporal.Duration.from('P43Y')),
  2019, 11, "M11", 18);
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from('2019-11-18').add({ years: -43 }),
  1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from('1977-02-18').add({ months: -3 }),
  1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from('1976-12-08').add({ days: -20 }),
  1976, 11, "M11", 18);
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from('2019-02-28').add({ months: -1 }),
  2019, 1, "M01", 28);

let p1y = new Temporal.Duration(1);
let p4y = new Temporal.Duration(4);
let p5m = new Temporal.Duration(0, 5);
let p1y2m = new Temporal.Duration(1, 2);
let p1y4d = new Temporal.Duration(1, 0, 0, 4);
let p1y2m4d = new Temporal.Duration(1, 2, 0, 4);
let p10d = new Temporal.Duration(0, 0, 0, 10);
let p1w = new Temporal.Duration(0, 0, 1);
let p6w = new Temporal.Duration(0, 0, 6);
let p2w3d = new Temporal.Duration(0, 0, 2, 3);
let p1y2w = new Temporal.Duration(1, 0, 2);
let p2m3w = new Temporal.Duration(0, 2, 3);

let testData = [
  [ '2020-02-29', p1y, '2021-02-28' ],
  [ '2020-02-29', p4y, '2024-02-29' ],
  [ '2021-07-16', p1y, '2022-07-16' ],
  [ '2021-07-16', p5m, '2021-12-16' ],
  [ '2021-08-16', p5m, '2022-01-16' ],
  [ '2021-10-31', p5m, '2022-03-31' ],
  [ '2021-09-30', p5m, '2022-02-28' ],
  [ '2019-09-30', p5m, '2020-02-29' ],
  [ '2019-10-01', p5m, '2020-03-01' ],
  [ '2021-07-16', p1y2m, '2022-09-16' ],
  [ '2021-11-30', p1y2m, '2023-01-30' ],
  [ '2021-12-31', p1y2m, '2023-02-28' ],
  [ '2022-12-31', p1y2m, '2024-02-29' ],
  [ '2021-07-16', p1y4d, '2022-07-20' ],
  [ '2021-02-27', p1y4d, '2022-03-03' ],
  [ '2023-02-27', p1y4d, '2024-03-02' ],
  [ '2021-12-30', p1y4d, '2023-01-03' ],
  [ '2021-07-30', p1y4d, '2022-08-03' ],
  [ '2021-06-30', p1y4d, '2022-07-04' ],
  [ '2021-07-16', p1y2m4d, '2022-09-20' ],
  [ '2021-02-27', p1y2m4d, '2022-05-01' ],
  [ '2021-02-26', p1y2m4d, '2022-04-30' ],
  [ '2023-02-26', p1y2m4d, '2024-04-30' ],
  [ '2021-12-30', p1y2m4d, '2023-03-04' ],
  [ '2021-07-30', p1y2m4d, '2022-10-04' ],
  [ '2021-06-30', p1y2m4d, '2022-09-03' ],
  [ '2021-07-16', p10d, '2021-07-26' ],
  [ '2021-07-26', p10d, '2021-08-05' ],
  [ '2021-12-26', p10d, '2022-01-05' ],
  [ '2020-02-26', p10d, '2020-03-07' ],
  [ '2021-02-26', p10d, '2021-03-08' ],
  [ '2020-02-19', p10d, '2020-02-29' ],
  [ '2021-02-19', p10d, '2021-03-01' ],
  [ '2021-02-19', p1w, '2021-02-26' ],
  [ '2021-02-27', p1w, '2021-03-06' ],
  [ '2020-02-27', p1w, '2020-03-05' ],
  [ '2021-12-24', p1w, '2021-12-31' ],
  [ '2021-12-27', p1w, '2022-01-03' ],
  [ '2021-01-27', p1w, '2021-02-03' ],
  [ '2021-06-27', p1w, '2021-07-04' ],
  [ '2021-07-27', p1w, '2021-08-03' ],
  [ '2021-02-19', p6w, '2021-04-02' ],
  [ '2021-02-27', p6w, '2021-04-10' ],
  [ '2020-02-27', p6w, '2020-04-09' ],
  [ '2021-12-24', p6w, '2022-02-04' ],
  [ '2021-12-27', p6w, '2022-02-07' ],
  [ '2021-01-27', p6w, '2021-03-10' ],
  [ '2021-06-27', p6w, '2021-08-08' ],
  [ '2021-07-27', p6w, '2021-09-07' ],
  [ '2020-02-29', p2w3d, '2020-03-17' ],
  [ '2020-02-28', p2w3d, '2020-03-16' ],
  [ '2021-02-28', p2w3d, '2021-03-17' ],
  [ '2020-12-28', p2w3d, '2021-01-14' ],
  [ '2020-02-29', p1y2w, '2021-03-14' ],
  [ '2020-02-28', p1y2w, '2021-03-14' ],
  [ '2021-02-28', p1y2w, '2022-03-14' ],
  [ '2020-12-28', p1y2w, '2022-01-11' ],
  [ '2020-02-29', p2m3w, '2020-05-20' ],
  [ '2020-02-28', p2m3w, '2020-05-19' ],
  [ '2021-02-28', p2m3w, '2021-05-19' ],
  [ '2020-12-28', p2m3w, '2021-03-21' ],
  [ '2019-12-28', p2m3w, '2020-03-20' ],
  [ '2019-10-28', p2m3w, '2020-01-18' ],
  [ '2019-10-31', p2m3w, '2020-01-21' ],
];

for (let [dateString, duration, resultString] of testData) {
  const date = Temporal.PlainDate.from(dateString);
  const result = Temporal.PlainDate.from(resultString);
  TemporalHelpers.assertPlainDatesEqual(date.add(duration), result);
}
