// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Incorrectly-spelled properties are ignored in relativeTo.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const oneMonth = Temporal.Duration.from({ months: 1 });

TemporalHelpers.assertDuration(oneMonth.round({largestUnit: "days",
                                               relativeTo: {
                                                   year: 2020,
                                                   month: 1,
                                                   day: 1,
                                                   months: 2
                                               }}),
                               0, 0, 0, 31, 0, 0, 0, 0, 0, 0);
 
