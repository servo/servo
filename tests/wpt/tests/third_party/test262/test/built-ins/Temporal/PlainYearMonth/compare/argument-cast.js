// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: compare() casts its arguments
features: [Temporal]
---*/

const nov94 = Temporal.PlainYearMonth.from("1994-11");
const jun13 = Temporal.PlainYearMonth.from("2013-06");

assert.sameValue(Temporal.PlainYearMonth.compare({ year: 1994, month: 11 }, jun13), -1, "one object");
assert.sameValue(Temporal.PlainYearMonth.compare("1994-11", jun13), -1, "one string");
assert.throws(TypeError, () => Temporal.PlainYearMonth.compare({ year: 1994 }, jun13), "one missing property");

assert.sameValue(Temporal.PlainYearMonth.compare(nov94, { year: 2013, month: 6 }), -1, "two object");
assert.sameValue(Temporal.PlainYearMonth.compare(nov94, "2013-06"), -1, "two string");
assert.throws(TypeError, () => Temporal.PlainYearMonth.compare(nov94, { year: 2013 }), "two missing property");
