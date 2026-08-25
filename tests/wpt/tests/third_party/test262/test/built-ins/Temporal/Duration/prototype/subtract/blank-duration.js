// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const blank1 = new Temporal.Duration();
const blank2 = new Temporal.Duration();

const result = blank1.subtract(blank2);

TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "result is also blank");
