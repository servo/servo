// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.valueof
description: Basic tests for valueOf().
features: [Temporal]
---*/

const instant = new Temporal.Instant(100n);
const instant2 = new Temporal.Instant(987654321n);

assert.throws(TypeError, () => instant.valueOf(), "valueOf");
assert.throws(TypeError, () => instant < instant, "<");
assert.throws(TypeError, () => instant <= instant, "<=");
assert.throws(TypeError, () => instant > instant, ">");
assert.throws(TypeError, () => instant >= instant, ">=");
assert.sameValue(instant === instant, true, "===");
assert.sameValue(instant === instant2, false, "===");
assert.sameValue(instant !== instant, false, "!==");
assert.sameValue(instant !== instant2, true, "!==");
