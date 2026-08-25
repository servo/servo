// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration
description: Test for Temporal.Duration subclassing.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

class CustomDuration extends Temporal.Duration {
}

const instance = new CustomDuration(1, 1, 0, 1);
TemporalHelpers.assertDuration(instance, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0);
assert.sameValue(Object.getPrototypeOf(instance), CustomDuration.prototype, "Instance of CustomDuration");
assert(instance instanceof CustomDuration, "Instance of CustomDuration");
assert(instance instanceof Temporal.Duration, "Instance of Temporal.Duration");
