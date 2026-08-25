// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: String arguments are supported.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const duration = Temporal.Duration.from({ days: 3, hours: 1, minutes: 10 });
const result = duration.subtract('P1DT5M');
TemporalHelpers.assertDuration(result, 0, 0, 0, 2, 1, 5, 0, 0, 0, 0, "String argument should be supported");
assert.throws(RangeError, () => duration.subtract("2DT5M"), "Invalid string argument should throw");
