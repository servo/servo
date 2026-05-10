// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Shorthand form is treated the same as options bag form with respect to
  incorrect strings
info: |
  1. If _directionParam_ is a String, then
    1. Let _paramString_ be _directionParam_.
    1. Set _roundTo_ to OrdinaryObjectCreate(*null*).
    1. Perform ! CreateDataPropertyOrThrow(_directionParam_, *"direction"*, _paramString_).
  ...
  1. Let _direction_ be ? GetDirectionOption(_directionParam_).
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

const badStrings = ['PREVIOUS', 'following', 'next\0', 'prevıous'];
for (const badString of badStrings) {
  assert.throws(RangeError, () => zdt.getTimeZoneTransition(badString));
  assert.throws(RangeError, () => zdt.getTimeZoneTransition({ direction: badString }));
}
