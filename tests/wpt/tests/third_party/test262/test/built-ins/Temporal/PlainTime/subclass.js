// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: Test for Temporal.PlainTime subclassing.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

class CustomPlainTime extends Temporal.PlainTime {
}

const instance = new CustomPlainTime(12, 34, 56, 987, 654, 321);
TemporalHelpers.assertPlainTime(instance, 12, 34, 56, 987, 654, 321);
assert.sameValue(Object.getPrototypeOf(instance), CustomPlainTime.prototype, "Instance of CustomPlainTime");
assert(instance instanceof CustomPlainTime, "Instance of CustomPlainTime");
assert(instance instanceof Temporal.PlainTime, "Instance of Temporal.PlainTime");
