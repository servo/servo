// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: Basic tests for equals()
features: [Temporal]
---*/

const md1 = Temporal.PlainMonthDay.from("01-22");
const md2 = Temporal.PlainMonthDay.from("12-15");
assert(md1.equals(md1), "same object");
assert.sameValue(md1.equals(md2), false, "different object");

assert(md1.equals("01-22"), "same string");
assert.sameValue(md2.equals("01-22"), false, "different string");

assert(md1.equals({ month: 1, day: 22 }), "same property bag");
assert.sameValue(md2.equals({ month: 1, day: 22 }), false, "different property bag");

assert.throws(TypeError, () => md1.equals({ month: 1 }), "missing field in property bag");

const mdYear1 = new Temporal.PlainMonthDay(1, 1, undefined, 1972);
const mdYear2 = new Temporal.PlainMonthDay(1, 1, undefined, 2000);
assert.sameValue(mdYear1.equals(mdYear2), false, "different reference years");
