// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Should round relative to the receiver.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date1 = Temporal.PlainDate.from("2019-01-01");
const date2 = Temporal.PlainDate.from("2019-02-15");

TemporalHelpers.assertDuration(
  date1.until(date2, { smallestUnit: "months", roundingMode: "halfExpand" }),
  0, /* months = */ 2, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
  date2.until(date1, { smallestUnit: "months", roundingMode: "halfExpand" }),
  0, /* months = */ -1, 0, 0, 0, 0, 0, 0, 0, 0);

const cases = [
  ["2019-03-01", "2019-01-29", 1, 1],
  ["2019-01-29", "2019-03-01", -1, -3],
  ["2019-03-29", "2019-01-30", 1, 29],
  ["2019-01-30", "2019-03-29", -1, -29],
  ["2019-03-30", "2019-01-31", 1, 30],
  ["2019-01-31", "2019-03-30", -1, -28],
  ["2019-03-31", "2019-01-31", 2, 0],
  ["2019-01-31", "2019-03-31", -2, 0]
];
for (const [end, start, months, days] of cases) {
  const result = Temporal.PlainDate.from(start).until(end, { largestUnit: "months" });
  TemporalHelpers.assertDuration(result, 0, months, 0, days, 0, 0, 0, 0, 0, 0, `${end} - ${start}`);
}
