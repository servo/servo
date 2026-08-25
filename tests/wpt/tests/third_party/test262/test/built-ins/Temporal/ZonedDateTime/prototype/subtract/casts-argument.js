// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Casts argument from string.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("1969-12-25T12:23:45.678901234+00:00[UTC]");

TemporalHelpers.assertZonedDateTimesEqual(
    zdt.subtract("PT240H0.000000800S"),
    Temporal.ZonedDateTime.from("1969-12-15T12:23:45.678900434+00:00[UTC]"));
