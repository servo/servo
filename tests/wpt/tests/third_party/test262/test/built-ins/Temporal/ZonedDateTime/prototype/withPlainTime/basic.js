// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: withPlainTime() works.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("2015-12-07T03:24:30.000003500[-08:00]");

// withPlainTime({ hour: 10 }) works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.withPlainTime({ hour: 10 }),
    Temporal.ZonedDateTime.from("2015-12-07T10:00:00-08:00[-08:00]"));

// withPlainTime(time) works
const time = new Temporal.PlainTime(11, 22);
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.withPlainTime(time),
    Temporal.ZonedDateTime.from("2015-12-07T11:22:00-08:00[-08:00]"));

// withPlainTime('12:34') works
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.withPlainTime("12:34"),
    Temporal.ZonedDateTime.from( "2015-12-07T12:34:00-08:00[-08:00]"));
