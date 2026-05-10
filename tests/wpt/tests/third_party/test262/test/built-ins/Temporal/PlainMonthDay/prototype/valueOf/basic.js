// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.valueof
description: Basic tests for valueOf().
features: [Temporal]
---*/

const plainMonthDay = Temporal.PlainMonthDay.from("1963-02-13");
const plainMonthDay2 = Temporal.PlainMonthDay.from("1963-02-13");

assert.throws(TypeError, () => plainMonthDay.valueOf(), "valueOf");
assert.throws(TypeError, () => plainMonthDay < plainMonthDay, "<");
assert.throws(TypeError, () => plainMonthDay <= plainMonthDay, "<=");
assert.throws(TypeError, () => plainMonthDay > plainMonthDay, ">");
assert.throws(TypeError, () => plainMonthDay >= plainMonthDay, ">=");
assert.sameValue(plainMonthDay === plainMonthDay, true, "===");
assert.sameValue(plainMonthDay === plainMonthDay2, false, "===");
assert.sameValue(plainMonthDay !== plainMonthDay, false, "!==");
assert.sameValue(plainMonthDay !== plainMonthDay2, true, "!==");
