// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Does not balance up to weeks if largestUnit is larger than weeks.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const monthAlmostWeek = new Temporal.Duration(0, 1, 0, 6, 20, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(monthAlmostWeek.round({
    smallestUnit: "days",
    relativeTo: new Temporal.PlainDate(2020, 1, 1)
}), 0, 1, 0, 7, 0, 0, 0, 0, 0, 0);
