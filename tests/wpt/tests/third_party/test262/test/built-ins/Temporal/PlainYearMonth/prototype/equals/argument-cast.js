// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: equals() casts its argument
features: [Temporal]
---*/

const nov94 = Temporal.PlainYearMonth.from("1994-11");

assert.sameValue(nov94.equals({ year: 2013, month: 6 }), false, "object");
assert.sameValue(nov94.equals({ year: 1994, month: 11 }), true, "object");
assert.sameValue(nov94.equals("2013-06"), false, "string");
assert.sameValue(nov94.equals("1994-11"), true, "string");
assert.throws(TypeError, () => nov94.equals({ year: 2013 }), "missing property");
