// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Can parse negative extended year
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("-009999-11-18T15:23:30.12+00:00[UTC]"),
    new Temporal.ZonedDateTime(-377677326989880000000n, "UTC"));
