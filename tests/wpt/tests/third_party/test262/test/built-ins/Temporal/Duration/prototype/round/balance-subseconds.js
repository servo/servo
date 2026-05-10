// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Balancing from subsecond units to seconds happens correctly
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const pos = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 999, 999999, 999999999);
TemporalHelpers.assertDuration(pos.round({ largestUnit: "seconds" }), 0, 0, 0, 0, 0, 0, 2, 998, 998, 999);

const neg = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -999, -999999, -999999999);
TemporalHelpers.assertDuration(neg.round({ largestUnit: "seconds" }), 0, 0, 0, 0, 0, 0, -2, -998, -998, -999);
