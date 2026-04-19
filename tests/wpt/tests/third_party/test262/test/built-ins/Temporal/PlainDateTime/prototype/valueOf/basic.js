// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.valueof
description: Comparison operators (except !== and ===) do not work
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(1963, 2, 13, 9, 36, 29, 123, 456, 789);
const dt1again = new Temporal.PlainDateTime(1963, 2, 13, 9, 36, 29, 123, 456, 789);
const dt2 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

assert.throws(TypeError, () => dt1.valueOf(), "always throws");
assert.sameValue(dt1 === dt1, true, "object equality implies ===");
assert.sameValue(dt1 !== dt1again, true, "object non-equality, even if all data is the same, implies !==");
assert.throws(TypeError, () => dt1 < dt1, "< throws (same objects)");
assert.throws(TypeError, () => dt1 < dt2, "< throws (different objects)");
assert.throws(TypeError, () => dt1 > dt1, "> throws (same objects)");
assert.throws(TypeError, () => dt1 > dt2, "> throws (different objects)");
assert.throws(TypeError, () => dt1 <= dt1, "<= does not throw (same objects)");
assert.throws(TypeError, () => dt1 <= dt2, "<= throws (different objects)");
assert.throws(TypeError, () => dt1 >= dt1, ">= throws (same objects)");
assert.throws(TypeError, () => dt1 >= dt2, ">= throws (different objects)");
