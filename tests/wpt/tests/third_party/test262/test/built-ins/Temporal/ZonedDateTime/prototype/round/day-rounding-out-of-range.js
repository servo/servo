// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: >
  Finding the upper bound for day rounding may fail if the instance is at the
  upper edge of the representable range
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(86400_0000_0000_000_000_000n, "UTC");
assert.throws(RangeError, () => instance.round({ smallestUnit: 'day' }), "Upper bound for rounding is out of range");
