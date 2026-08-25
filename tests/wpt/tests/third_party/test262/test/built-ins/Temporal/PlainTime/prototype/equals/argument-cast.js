// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: equals() casts its argument
features: [Temporal]
---*/

const t1 = Temporal.PlainTime.from("08:44:15.321");

assert.sameValue(t1.equals({ hour: 14, minute: 23, second: 30, millisecond: 123 }), false, "object");
assert.sameValue(t1.equals({ hour: 8, minute: 44, second: 15, millisecond: 321 }), true, "object");
assert.sameValue(t1.equals("14:23:30.123"), false, "string");
assert.sameValue(t1.equals("08:44:15.321"), true, "string");
assert.throws(TypeError, () => t1.equals({}), "no properties");
assert.throws(TypeError, () => t1.equals({ hours: 8 }), "only plural property");
