// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Balances up to the next unit after rounding.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const almostWeek = new Temporal.Duration(0, 0, 0, 6, 20, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(almostWeek.round({
    largestUnit: "weeks",
    smallestUnit: "days",
    relativeTo: new Temporal.PlainDate(2020, 1, 1)
}), 0, 0, 1, 0, 0, 0, 0, 0, 0, 0);
