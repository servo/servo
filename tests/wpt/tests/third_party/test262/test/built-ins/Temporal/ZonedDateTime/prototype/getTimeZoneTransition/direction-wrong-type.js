// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: Value of direction property cannot be a primitive other than string
info: |
  1. Let _direction_ be ? GetDirectionOption(_directionParam_).
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

const rangeErrorValues = [false, 42, 55n, null];
for (const badValue of rangeErrorValues) {
  assert.throws(RangeError, () => zdt.getTimeZoneTransition({ direction: badValue }), "Non-Symbol throws a RangeError");
}
assert.throws(TypeError, () => zdt.getTimeZoneTransition({ direction: Symbol("next") }), "Symbol throws a TypeError");
