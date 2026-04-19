// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: Test for Temporal.PlainYearMonth subclassing.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

class CustomPlainYearMonth extends Temporal.PlainYearMonth {
}

const instance = new CustomPlainYearMonth(2000, 5);
TemporalHelpers.assertPlainYearMonth(instance, 2000, 5, "M05");
assert.sameValue(Object.getPrototypeOf(instance), CustomPlainYearMonth.prototype, "Instance of CustomPlainYearMonth");
assert(instance instanceof CustomPlainYearMonth, "Instance of CustomPlainYearMonth");
assert(instance instanceof Temporal.PlainYearMonth, "Instance of Temporal.PlainYearMonth");
