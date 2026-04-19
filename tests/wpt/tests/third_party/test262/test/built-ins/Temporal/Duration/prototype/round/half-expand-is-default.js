// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: halfExpand is the default.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const relativeTo = new Temporal.PlainDate(2020, 1, 1);

TemporalHelpers.assertDuration(d.round({
    smallestUnit: "years",
    relativeTo
}), 6, 0, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(d.negated().round({
    smallestUnit: "years",
    relativeTo
}), -6, 0, 0, 0, 0, 0, 0, 0, 0, 0);
