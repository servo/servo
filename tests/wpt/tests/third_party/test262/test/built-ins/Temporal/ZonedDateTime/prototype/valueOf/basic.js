// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.valueof
description: Basic tests for valueOf().
features: [Temporal]
---*/

const zonedDateTime = new Temporal.ZonedDateTime(100n, "UTC");
const zonedDateTime2 = new Temporal.ZonedDateTime(987654321n, "UTC");

assert.throws(TypeError, () => zonedDateTime.valueOf(), "valueOf");
assert.throws(TypeError, () => zonedDateTime < zonedDateTime, "<");
assert.throws(TypeError, () => zonedDateTime <= zonedDateTime, "<=");
assert.throws(TypeError, () => zonedDateTime > zonedDateTime, ">");
assert.throws(TypeError, () => zonedDateTime >= zonedDateTime, ">=");
assert.sameValue(zonedDateTime === zonedDateTime, true, "===");
assert.sameValue(zonedDateTime === zonedDateTime2, false, "===");
assert.sameValue(zonedDateTime !== zonedDateTime, false, "!==");
assert.sameValue(zonedDateTime !== zonedDateTime2, true, "!==");
