// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.valueof
description: Basic tests for valueOf().
features: [Temporal]
---*/

const plainTime = Temporal.PlainTime.from("09:36:29.123456789");
const plainTime2 = Temporal.PlainTime.from("09:36:29.123456789");

assert.throws(TypeError, () => plainTime.valueOf(), "valueOf");
assert.throws(TypeError, () => plainTime < plainTime, "<");
assert.throws(TypeError, () => plainTime <= plainTime, "<=");
assert.throws(TypeError, () => plainTime > plainTime, ">");
assert.throws(TypeError, () => plainTime >= plainTime, ">=");
assert.sameValue(plainTime === plainTime, true, "===");
assert.sameValue(plainTime === plainTime2, false, "===");
assert.sameValue(plainTime !== plainTime, false, "!==");
assert.sameValue(plainTime !== plainTime2, true, "!==");
