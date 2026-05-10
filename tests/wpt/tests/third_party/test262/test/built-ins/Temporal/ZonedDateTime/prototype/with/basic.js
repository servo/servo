// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: with() works on various values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");

// zdt.with({ year: 2019 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ year: 2019 }),
    Temporal.ZonedDateTime.from("2019-11-18T15:23:30.123456789+00:00[UTC]"));

// zdt.with({ month: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ month: 5 }),
    Temporal.ZonedDateTime.from("1976-05-18T15:23:30.123456789+00:00[UTC]"));

// zdt.with({ monthCode: "M05" }) works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ monthCode: "M05" }),
    Temporal.ZonedDateTime.from("1976-05-18T15:23:30.123456789+00:00[UTC]"));

// zdt.with({ day: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ day: 5 }),
    Temporal.ZonedDateTime.from("1976-11-05T15:23:30.123456789+00:00[UTC]"));

// zdt.with({ hour: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ hour: 5 }),
    Temporal.ZonedDateTime.from("1976-11-18T05:23:30.123456789+00:00[UTC]"));

// zdt.with({ minute: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ minute: 5 }),
    Temporal.ZonedDateTime.from("1976-11-18T15:05:30.123456789+00:00[UTC]"));

// zdt.with({ second: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ second: 5 }),
    Temporal.ZonedDateTime.from("1976-11-18T15:23:05.123456789+00:00[UTC]"));

// zdt.with({ millisecond: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ millisecond: 5 }),
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.005456789+00:00[UTC]"));

// zdt.with({ microsecond: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ microsecond: 5 }),
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123005789+00:00[UTC]"));

// zdt.with({ nanosecond: 5 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ nanosecond: 5 }),
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456005+00:00[UTC]"));

// zdt.with({ month: 5, second: 15 } works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({
        month: 5,
        second: 15
    }),
    Temporal.ZonedDateTime.from("1976-05-18T15:23:15.123456789+00:00[UTC]"));
