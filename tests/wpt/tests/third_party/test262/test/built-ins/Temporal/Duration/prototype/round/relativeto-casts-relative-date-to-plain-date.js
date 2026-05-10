// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: round() casts relativeTo to PlainDate if possible
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const hours25 = new Temporal.Duration(0, 0, 0, 0, 25, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(hours25.round({ largestUnit: "days",
                                               relativeTo: "2019-11-02"
                                             }),
                               0, 0, 0, 1, 1, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(hours25.round({ largestUnit: "days",
                                               relativeTo: {
                                                   year: 2019,
                                                   month: 11,
                                                   day: 2
                                               }}),
                               0, 0, 0, 1, 1, 0, 0, 0, 0, 0);
