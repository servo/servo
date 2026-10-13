// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Always rounds to a startOfDay, even when midnight occurs twice
info: |
  https://github.com/tc39/proposal-temporal/issues/2938
  https://github.com/tc39/proposal-temporal/issues/3312
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt1 = Temporal.ZonedDateTime.from('2010-03-04T23:10:00+11:00[Antarctica/Casey]');
const zdt2 = Temporal.ZonedDateTime.from('2010-03-05T00:45:00+11:00[Antarctica/Casey]');
// 3h backwards shift from +11 to +08 at 2010-03-05 02:00
const zdt3 = Temporal.ZonedDateTime.from('2010-03-04T23:10:00+08:00[Antarctica/Casey]');
const zdt4 = Temporal.ZonedDateTime.from('2010-03-05T00:45:00+08:00[Antarctica/Casey]');

const startOfMarch4 = Temporal.ZonedDateTime.from("2010-03-04T00:00:00+11:00[Antarctica/Casey]");
const startOfMarch5 = Temporal.ZonedDateTime.from("2010-03-05T00:00:00+11:00[Antarctica/Casey]");
const startOfMarch6 = Temporal.ZonedDateTime.from("2010-03-06T00:00:00+08:00[Antarctica/Casey]");

// Rounding down

for (const roundingMode of ["floor", "trunc"]) {
  const options = { smallestUnit: "days", roundingMode };
  TemporalHelpers.assertZonedDateTimesEqual(zdt1.round(options), startOfMarch4, `${zdt1} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt2.round(options), startOfMarch5, `${zdt2} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt3.round(options), startOfMarch4, `${zdt3} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt4.round(options), startOfMarch5, `${zdt4} ${roundingMode}`);
}

// Rounding to nearest

for (const roundingMode of ["halfCeil", "halfEven", "halfExpand", "halfFloor", "halfTrunc"]) {
  const options = { smallestUnit: "days", roundingMode };
  TemporalHelpers.assertZonedDateTimesEqual(zdt1.round(options), startOfMarch5, `${zdt1} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt2.round(options), startOfMarch5, `${zdt2} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt3.round(options), startOfMarch5, `${zdt3} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt4.round(options), startOfMarch5, `${zdt4} ${roundingMode}`);
}

// Rounding up

for (const roundingMode of ["ceil", "expand"]) {
  const options = { smallestUnit: "days", roundingMode };
  TemporalHelpers.assertZonedDateTimesEqual(zdt1.round(options), startOfMarch5, `${zdt1} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt2.round(options), startOfMarch6, `${zdt2} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt3.round(options), startOfMarch5, `${zdt3} ${roundingMode}`);
  TemporalHelpers.assertZonedDateTimesEqual(zdt4.round(options), startOfMarch6, `${zdt4} ${roundingMode}`);
}
