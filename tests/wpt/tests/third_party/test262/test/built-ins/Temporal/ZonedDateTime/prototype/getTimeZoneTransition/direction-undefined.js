// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: If using options bag form, direction property is required
info: |
  1. Let _direction_ be ? GetDirectionOption(_directionParam_).
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");
assert.throws(RangeError, () => zdt.getTimeZoneTransition({}));
assert.throws(RangeError, () => zdt.getTimeZoneTransition({ direction: undefined }));
assert.throws(RangeError, () => zdt.getTimeZoneTransition(() => {}));
