// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: Basic tests for compare()
features: [Temporal]
---*/

const nov94 = Temporal.PlainYearMonth.from("1994-11");
const nov94bis = Temporal.PlainYearMonth.from("1994-11");
const jun13 = Temporal.PlainYearMonth.from("2013-06");
assert.sameValue(Temporal.PlainYearMonth.compare(nov94, nov94), 0, "same object");
assert.sameValue(Temporal.PlainYearMonth.compare(nov94, nov94bis), 0, "different object");
assert.sameValue(Temporal.PlainYearMonth.compare(nov94, jun13), -1, "before");
assert.sameValue(Temporal.PlainYearMonth.compare(jun13, nov94), 1, "after");
