// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Basic tests for PlainMonthDay.from(object) with missing properties.
features: [Temporal]
---*/

assert.throws(TypeError, () => Temporal.PlainMonthDay.from({}), "No properties");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ day: 15 }), "Only day");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ month: 12 }), "day is required with month");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ monthCode: 'M12' }), "Only monthCode");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ monthCode: undefined, day: 15 }), "monthCode undefined");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ months: 12, day: 31 }), "months plural");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ year: 2021, month: 12 }), "day is required with year and month");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ year: 2021, monthCode: "M12" }), "day is required with year and monthCode");
assert.throws(TypeError, () => Temporal.PlainMonthDay.from({ year: 2021, day: 17 }), "either month or monthCode is required");
