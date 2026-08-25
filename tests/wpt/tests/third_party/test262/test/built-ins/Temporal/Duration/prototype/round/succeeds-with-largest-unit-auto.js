// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: round() succeeds with largestUnit 'auto'.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const hours25 = new Temporal.Duration(0, 0, 0, 0, 25, 0, 0, 0, 0, 0);

TemporalHelpers.assertDuration(hours25.round({ largestUnit: "auto" }),
                               0, 0, 0, 0, 25, 0, 0, 0, 0, 0);
