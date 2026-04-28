// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Days are 24 hours if relativeTo is ZonedDateTime and duration encompasses no DST change
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const hours25 = new Temporal.Duration(0, 0, 0, 0, 25, 0, 0, 0, 0, 0);
const relativeTo = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "+04:30");

TemporalHelpers.assertDuration(hours25.round({ largestUnit: "days", relativeTo }),
                               0, 0, 0, 1, 1, 0, 0, 0, 0, 0);
