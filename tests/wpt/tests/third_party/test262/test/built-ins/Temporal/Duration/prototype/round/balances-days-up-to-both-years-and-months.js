// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Balances days up to both years and months.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const twoYears = new Temporal.Duration(0, 11, 0, 396, 0, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(twoYears.round({
    largestUnit: "years",
    relativeTo: new Temporal.PlainDate(2017, 1, 1)
}), 2, 0, 0, 0, 0, 0, 0, 0, 0, 0);
