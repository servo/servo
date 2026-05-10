// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: String arguments are supported.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = Temporal.Duration.from({ days: 1, minutes: 5 });
const result = duration.add("P2DT5M");
TemporalHelpers.assertDuration(result, 0, 0, 0, 3, 0, 10, 0, 0, 0, 0, "String argument should be supported");
assert.throws(RangeError, () => duration.add("2DT5M"), "Invalid string argument should throw");
