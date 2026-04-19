// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Overflow options work.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");

const overflow = "constrain";
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ month: 29 }, { overflow }),
    Temporal.ZonedDateTime.from("1976-12-18T15:23:30.123456789+00:00[UTC]"));
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ day: 31 }, { overflow }),
    Temporal.ZonedDateTime.from("1976-11-30T15:23:30.123456789+00:00[UTC]"));
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ hour: 29 }, { overflow }),
    Temporal.ZonedDateTime.from("1976-11-18T23:23:30.123456789+00:00[UTC]"));
TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({ nanosecond: 9000 }, { overflow }),
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456999+00:00[UTC]"));
