// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.valueof
description: Basic tests for valueOf().
features: [Temporal]
---*/

const d1 = Temporal.Duration.from("P3DT1H");
const d2 = Temporal.Duration.from("P3DT1H");

assert.throws(TypeError, () => d1.valueOf(), "valueOf");
assert.throws(TypeError, () => d1 < d1, "<");
assert.throws(TypeError, () => d1 <= d1, "<=");
assert.throws(TypeError, () => d1 > d1, ">");
assert.throws(TypeError, () => d1 >= d1, ">=");
assert.sameValue(d1 === d1, true, "===");
assert.sameValue(d1 === d2, false, "===");
assert.sameValue(d1 !== d1, false, "!==");
assert.sameValue(d1 !== d2, true, "!==");
