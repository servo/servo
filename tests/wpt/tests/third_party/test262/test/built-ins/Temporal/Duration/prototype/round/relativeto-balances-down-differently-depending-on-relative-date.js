// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Balances down differently depending on relativeTo.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const oneYear = new Temporal.Duration(1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(oneYear.round({
    largestUnit: "days",
    relativeTo: new Temporal.PlainDate(2019, 1, 1)
}), 0, 0, 0, 365, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(oneYear.round({
    largestUnit: "days",
    relativeTo: new Temporal.PlainDate(2019, 7, 1)
}), 0, 0, 0, 366, 0, 0, 0, 0, 0, 0);

const minusYear = new Temporal.Duration(-1, 0, 0, 0, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(minusYear.round({
    largestUnit: "days",
    relativeTo: new Temporal.PlainDate(2020, 1, 1)
}), 0, 0, 0, -365, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(minusYear.round({
    largestUnit: "days",
    relativeTo: new Temporal.PlainDate(2020, 7, 1)
}), 0, 0, 0, -366, 0, 0, 0, 0, 0, 0);
