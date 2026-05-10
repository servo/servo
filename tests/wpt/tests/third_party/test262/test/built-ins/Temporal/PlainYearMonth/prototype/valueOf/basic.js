// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.valueof
description: Basic tests for valueOf().
features: [Temporal]
---*/

const plainYearMonth = Temporal.PlainYearMonth.from("1963-02");
const plainYearMonth2 = Temporal.PlainYearMonth.from("1963-02");

assert.throws(TypeError, () => plainYearMonth.valueOf(), "valueOf");
assert.throws(TypeError, () => plainYearMonth < plainYearMonth, "<");
assert.throws(TypeError, () => plainYearMonth <= plainYearMonth, "<=");
assert.throws(TypeError, () => plainYearMonth > plainYearMonth, ">");
assert.throws(TypeError, () => plainYearMonth >= plainYearMonth, ">=");
assert.sameValue(plainYearMonth === plainYearMonth, true, "===");
assert.sameValue(plainYearMonth === plainYearMonth2, false, "===");
assert.sameValue(plainYearMonth !== plainYearMonth, false, "!==");
assert.sameValue(plainYearMonth !== plainYearMonth2, true, "!==");
