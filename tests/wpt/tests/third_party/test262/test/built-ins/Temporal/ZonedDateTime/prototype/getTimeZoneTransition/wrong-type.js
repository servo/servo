// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Options bag cannot be anything other than a string, an object, or undefined
info: |
  1. If _directionParam_ is a String, then
    ...
  1. Else,
    1. Set _directionParam_ to ? GetOptionsObject(_directionParam_).
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

const badValues = [false, 42, 55n, Symbol("foo"), null];
for (const badValue of badValues) {
  assert.throws(TypeError, () => zdt.getTimeZoneTransition(badValue));
}
