// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.with
description: Positive and negative values in the temporalDurationLike argument are not acceptable
features: [Temporal]
---*/

const instance = new Temporal.Duration(0, 0, 0, 1, 2, 3, 4, 987, 654, 321);

assert.throws(
  RangeError,
  () => instance.with({ hours: 1, minutes: -30 }),
  `mixed positive and negative values always throw`
);
