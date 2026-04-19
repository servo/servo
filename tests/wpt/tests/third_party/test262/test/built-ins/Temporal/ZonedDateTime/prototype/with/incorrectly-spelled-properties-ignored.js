// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Incorrectly-spelled properties are ignored.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");

TemporalHelpers.assertZonedDateTimesEqual(
    zdt.with({
        month: 12,
        days: 15
    }),
    Temporal.ZonedDateTime.from("1976-12-18T15:23:30.123456789+00:00[UTC]"));
