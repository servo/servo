// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: >
  BalanceDuration throws a RangeError when the result is too large.
features: [Temporal]
---*/

// Math.trunc(Number.MAX_SAFE_INTEGER / 86400) === 104249991374
var duration1 = Temporal.Duration.from({days: 104249991374});
var duration2 = Temporal.Duration.from({days: -104249991374});

assert.throws(RangeError, () => duration1.subtract(duration2));
