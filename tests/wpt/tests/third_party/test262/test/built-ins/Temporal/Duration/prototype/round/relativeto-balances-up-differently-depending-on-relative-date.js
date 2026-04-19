// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Balances up differently depending on relativeTo.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const fortyDays = new Temporal.Duration(0, 0, 0, 40, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(fortyDays.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 1, 1)
}), 0, 1, 0, 9, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(fortyDays.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 2, 1)
}), 0, 1, 0, 11, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(fortyDays.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 3, 1)
}), 0, 1, 0, 9, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(fortyDays.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 4, 1)
}), 0, 1, 0, 10, 0, 0, 0, 0, 0, 0);

const minusForty = new Temporal.Duration(0, 0, 0, -40, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(minusForty.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 1, 1)
}), 0, -1, 0, -9, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(minusForty.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 2, 1)
}), 0, -1, 0, -9, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(minusForty.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 3, 1)
}), 0, -1, 0, -11, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(minusForty.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2020, 4, 1)
}), 0, -1, 0, -9, 0, 0, 0, 0, 0, 0);
