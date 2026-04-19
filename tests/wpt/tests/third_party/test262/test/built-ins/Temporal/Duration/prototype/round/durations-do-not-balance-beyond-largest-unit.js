// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Durations do not balance beyond their current largest unit by default.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const fortyDays = new Temporal.Duration(0, 0, 0, 40, 0, 0, 0, 0, 0, 0);
const one85Minutes = new Temporal.Duration(0, 0, 0, 0, 185, 0, 0, 0, 0, 0);
const relativeTo = new Temporal.PlainDate(2020, 1, 1);

TemporalHelpers.assertDuration(fortyDays.round({
    smallestUnit: "seconds",
    relativeTo
}), 0, 0, 0, 40, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(one85Minutes.round({
    smallestUnit: "seconds",
}), 0, 0, 0, 0, 185, 0, 0, 0, 0, 0);
