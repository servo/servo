// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: Basic tests for equals()
features: [Temporal]
---*/

const nov94 = Temporal.PlainYearMonth.from("1994-11");
const nov94bis = Temporal.PlainYearMonth.from("1994-11");
const jun13 = Temporal.PlainYearMonth.from("2013-06");
assert.sameValue(nov94.equals(nov94), true, "same object");
assert.sameValue(nov94.equals(nov94bis), true, "different object");
assert.sameValue(nov94.equals(jun13), false, "different year-months");
