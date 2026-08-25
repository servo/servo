// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate
description: Test for Temporal.PlainDate subclassing.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

class CustomPlainDate extends Temporal.PlainDate {
}

const instance = new CustomPlainDate(2000, 5, 2);
TemporalHelpers.assertPlainDate(instance, 2000, 5, "M05", 2);
assert.sameValue(Object.getPrototypeOf(instance), CustomPlainDate.prototype, "Instance of CustomPlainDate");
assert(instance instanceof CustomPlainDate, "Instance of CustomPlainDate");
assert(instance instanceof Temporal.PlainDate, "Instance of Temporal.PlainDate");
