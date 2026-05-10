// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: Incorrectly-spelled properties are ignored.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("2015-12-07T03:24:30.000003500[-08:00]");

TemporalHelpers.assertZonedDateTimesEqual(
    zdt.withPlainTime({
        hour: 10,
        seconds: 55
    }),
    Temporal.ZonedDateTime.from("2015-12-07T10:00:00-08:00[-08:00]"));
