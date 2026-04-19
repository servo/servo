// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
description: Test for Temporal.PlainDateTime subclassing.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

class CustomPlainDateTime extends Temporal.PlainDateTime {
}

const instance = new CustomPlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
TemporalHelpers.assertPlainDateTime(instance, 2000, 5, "M05", 2, 12, 34, 56, 987, 654, 321);
assert.sameValue(Object.getPrototypeOf(instance), CustomPlainDateTime.prototype, "Instance of CustomPlainDateTime");
assert(instance instanceof CustomPlainDateTime, "Instance of CustomPlainDateTime");
assert(instance instanceof Temporal.PlainDateTime, "Instance of Temporal.PlainDateTime");
