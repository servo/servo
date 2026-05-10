// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Can parse variant decimal separator.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from("1976-11-18T15:23:30,12-08:00[-08:00]"),
    new Temporal.ZonedDateTime(217207410120000000n, "-08:00"));
