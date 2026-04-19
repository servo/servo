// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: from() can parse any number of decimal places
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.1-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410100000000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.12-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410120000000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123000000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.1234-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123400000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.12345-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123450000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123456000n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.1234567-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123456700n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.12345678-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123456780n, "-08:00"));
TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410123456789n, "-08:00"));
