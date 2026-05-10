// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: Test for Temporal.PlainMonthDay subclassing.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

class CustomPlainMonthDay extends Temporal.PlainMonthDay {
}

const instance = new CustomPlainMonthDay(5, 2);
TemporalHelpers.assertPlainMonthDay(instance, "M05", 2);
assert.sameValue(Object.getPrototypeOf(instance), CustomPlainMonthDay.prototype, "Instance of CustomPlainMonthDay");
assert(instance instanceof CustomPlainMonthDay, "Instance of CustomPlainMonthDay");
assert(instance instanceof Temporal.PlainMonthDay, "Instance of Temporal.PlainMonthDay");
