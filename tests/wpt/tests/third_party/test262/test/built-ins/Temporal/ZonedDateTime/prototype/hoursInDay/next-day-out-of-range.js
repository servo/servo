// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.hoursinday
description: >
  Finding the boundary with the next day may throw if the instance is at the
  upper edge of the representable range
features: [Temporal]
---*/

const z = new Temporal.ZonedDateTime(86400_0000_0000_000_000_000n, "UTC");
assert.throws(RangeError, () => z.hoursInDay, "Next day boundary is out of range");
