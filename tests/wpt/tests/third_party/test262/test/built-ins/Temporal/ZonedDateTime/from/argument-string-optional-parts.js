// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Optional parts.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30-08[-08:00]"),
    new Temporal.ZonedDateTime(217207410000000000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217206000000000000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("2020-01-01[+09:00]"),
    new Temporal.ZonedDateTime(1577804400000000000n, "+09:00"));
