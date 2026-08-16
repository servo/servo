// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.hoursinday
description: Handles dates with offset transitions where midnight occurs twice.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Test date with offset transition where the same day starts twice
// See https://github.com/tc39/proposal-temporal/issues/2938 for more details

const zdt1 = Temporal.ZonedDateTime.from('2010-11-06T00:00:00-02:30[America/St_Johns]');
const zdt2 = Temporal.ZonedDateTime.from('2010-11-07T23:00:00-03:30[America/St_Johns]');
const zdt3 = Temporal.ZonedDateTime.from('2010-11-08T23:00:00-03:30[America/St_Johns]');

assert.sameValue(zdt1.hoursInDay, 24);
assert.sameValue(zdt2.hoursInDay, 25);
assert.sameValue(zdt3.hoursInDay, 24);

const zdt4 = Temporal.ZonedDateTime.from("2010-03-04T23:10:00+11:00[Antarctica/Casey]");
const zdt5 = Temporal.ZonedDateTime.from("2010-03-05T00:45:00+11:00[Antarctica/Casey]");
// 3h backwards shift from +11 to +08 at 2010-03-05 02:00
const zdt6 = Temporal.ZonedDateTime.from("2010-03-04T23:10:00+08:00[Antarctica/Casey]");
const zdt7 = Temporal.ZonedDateTime.from("2010-03-05T00:45:00+08:00[Antarctica/Casey]");

assert.sameValue(zdt4.hoursInDay, 24, "March 4 has 24 hours (excluding discontiguous piece)");
assert.sameValue(zdt5.hoursInDay, 27, "March 5 has 27 hours (calculated from discontiguous piece)");
assert.sameValue(zdt6.hoursInDay, 24, "March 4 has 24 hours (calculated from discontiguous piece)");
assert.sameValue(zdt7.hoursInDay, 27, "March 5 has 27 hours (including discontiguous piece of March 4)");

const startOfMarch4 = Temporal.ZonedDateTime.from("2010-03-04T00:00:00+11:00[Antarctica/Casey]");
const startOfMarch5 = Temporal.ZonedDateTime.from("2010-03-05T00:00:00+11:00[Antarctica/Casey]");

TemporalHelpers.assertZonedDateTimesEqual(
  startOfMarch4.add({ hours: startOfMarch4.hoursInDay }),
  startOfMarch5,
  "start of day + hours in day = start of next day"
);
