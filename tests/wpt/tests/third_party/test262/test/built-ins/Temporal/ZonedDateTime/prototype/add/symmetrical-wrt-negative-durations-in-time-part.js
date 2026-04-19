// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Results are symmetrical with regard to negative durations in the time part.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// "2020-01-31T15:00-08:00[-08:00]"
const jan31 = new Temporal.ZonedDateTime(1580511600000000000n, "-08:00");
// "2020-01-31T14:30:00-08:00[-08:00]"
const expected1 = new Temporal.ZonedDateTime(1580509800000000000n, "-08:00");
// "2020-01-31T14:59:30-08:00[-08:00]"
const expected2 = new Temporal.ZonedDateTime(1580511570000000000n, "-08:00");

TemporalHelpers.assertZonedDateTimesEqual(jan31.add({ minutes: -30 }), expected1);
TemporalHelpers.assertZonedDateTimesEqual(jan31.add({ seconds: -30 }), expected2);
