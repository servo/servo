// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Symmetrical with regard to negative durations in the time part.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const mar31 = Temporal.ZonedDateTime.from("2020-03-31T15:00+00:00[UTC]");

TemporalHelpers.assertZonedDateTimesEqual(
    mar31.subtract({ minutes: -30 }),
    Temporal.ZonedDateTime.from("2020-03-31T15:30:00+00:00[UTC]"));

TemporalHelpers.assertZonedDateTimesEqual(
    mar31.subtract({ seconds: -30 }),
    Temporal.ZonedDateTime.from("2020-03-31T15:00:30+00:00[UTC]"));
