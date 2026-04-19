// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: compare() casts its arguments
features: [Temporal]
---*/

const t1 = Temporal.PlainTime.from("08:44:15.321");
const t2 = Temporal.PlainTime.from("14:23:30.123");

assert.sameValue(Temporal.PlainTime.compare({ hour: 16, minute: 34 }, t2), 1, "one object");
assert.sameValue(Temporal.PlainTime.compare("16:34", t2), 1, "one string");
assert.throws(TypeError, () => Temporal.PlainTime.compare({ hours: 16 }, t2), "one missing property");

assert.sameValue(Temporal.PlainTime.compare(t1, { hour: 16, minute: 34 }), -1, "two object");
assert.sameValue(Temporal.PlainTime.compare(t1, "16:34"), -1, "two string");
assert.throws(TypeError, () => Temporal.PlainTime.compare(t1, { hours: 16 }), "two missing property");
