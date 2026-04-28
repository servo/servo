// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: basic object coercion in arguments
features: [Temporal]
---*/

const d1 = Temporal.PlainDate.from("1976-11-18");
const d2 = Temporal.PlainDate.from("2019-06-30");

assert.sameValue(Temporal.PlainDate.compare({ year: 1976, month: 11, day: 18 }, d2), -1, "first argument");
assert.sameValue(Temporal.PlainDate.compare({ year: 2019, month: 6, day: 30 }, d2), 0, "first argument");
assert.sameValue(Temporal.PlainDate.compare({ year: 2024, month: 1, day: 12 }, d2), 1, "first argument");

assert.sameValue(Temporal.PlainDate.compare(d1, { year: 2024, month: 1, day: 12 }), -1, "second argument");
assert.sameValue(Temporal.PlainDate.compare(d1, { year: 1976, month: 11, day: 18 }), 0, "second argument");
assert.sameValue(Temporal.PlainDate.compare(d1, { year: 1926, month: 7, day: 7 }), 1, "second argument");

assert.throws(TypeError, () => Temporal.PlainDate.compare({ year: 1976 }, d2), "only year in first argument");
assert.throws(TypeError, () => Temporal.PlainDate.compare(d1, { year: 2019 }), "only year in second argument");
