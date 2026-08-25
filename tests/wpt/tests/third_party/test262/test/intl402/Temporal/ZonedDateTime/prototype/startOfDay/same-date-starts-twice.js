// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.startofday
description: Handles dates with offset transitions where midnight occurs twice.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Test date with offset transition where the same day starts twice
// See https://github.com/tc39/proposal-temporal/issues/2938 for more details

const zdt1 = Temporal.ZonedDateTime.from('2010-11-06T00:00:00-02:30[America/St_Johns]');
const zdt2 = Temporal.ZonedDateTime.from('2010-11-07T23:00:00-03:30[America/St_Johns]');
const zdt3 = Temporal.ZonedDateTime.from('2010-11-08T23:00:00-03:30[America/St_Johns]');

const startOfDay2 = Temporal.ZonedDateTime.from('2010-11-07T00:00:00-02:30[America/St_Johns]');
const startOfDay3 = Temporal.ZonedDateTime.from('2010-11-08T00:00:00-03:30[America/St_Johns]');

TemporalHelpers.assertZonedDateTimesEqual(zdt1.startOfDay(), zdt1);
TemporalHelpers.assertZonedDateTimesEqual(zdt2.startOfDay(), startOfDay2);
TemporalHelpers.assertZonedDateTimesEqual(zdt3.startOfDay(), startOfDay3);

const zdt4 = Temporal.ZonedDateTime.from("2010-03-04T23:10:00+11:00[Antarctica/Casey]");
const zdt5 = Temporal.ZonedDateTime.from("2010-03-05T00:45:00+11:00[Antarctica/Casey]");
// 3h backwards shift from +11 to +08 at 2010-03-05 02:00
const zdt6 = Temporal.ZonedDateTime.from("2010-03-04T23:10:00+08:00[Antarctica/Casey]");
const zdt7 = Temporal.ZonedDateTime.from("2010-03-05T00:45:00+08:00[Antarctica/Casey]");

const startOfMarch4 = Temporal.ZonedDateTime.from("2010-03-04T00:00:00+11:00[Antarctica/Casey]");
const startOfMarch5 = Temporal.ZonedDateTime.from("2010-03-05T00:00:00+11:00[Antarctica/Casey]");

TemporalHelpers.assertZonedDateTimesEqual(zdt4.startOfDay(), startOfMarch4, "start of March 4");
TemporalHelpers.assertZonedDateTimesEqual(zdt5.startOfDay(), startOfMarch5, "start of March 5, calculated from discontiguous piece");
TemporalHelpers.assertZonedDateTimesEqual(zdt6.startOfDay(), startOfMarch4, "start of March 4, calculated from discontiguous piece");
TemporalHelpers.assertZonedDateTimesEqual(zdt7.startOfDay(), startOfMarch5, "start of March 5");
