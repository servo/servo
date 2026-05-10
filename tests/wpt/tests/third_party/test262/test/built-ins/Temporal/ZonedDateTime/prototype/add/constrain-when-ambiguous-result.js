// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Constrains result when ambiguous.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// "2020-01-31T15:00-08:00[-08:00]"
const jan31 = new Temporal.ZonedDateTime(1580511600000000000n, "-08:00");
// "2020-02-29T15:00:00-08:00[-08:00]"
const expected = new Temporal.ZonedDateTime(1583017200000000000n, "-08:00");

TemporalHelpers.assertZonedDateTimesEqual(jan31.add({ months: 1 }), expected);
TemporalHelpers.assertZonedDateTimesEqual(
    jan31.add({ months: 1 }, { overflow: "constrain" }), expected);
