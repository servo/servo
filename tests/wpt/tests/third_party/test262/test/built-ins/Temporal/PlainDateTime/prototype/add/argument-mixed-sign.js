// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Positive and negative values in the temporalDurationLike argument are not acceptable
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 15, 30, 45, 987, 654, 321);

["constrain", "reject"].forEach((overflow) => {
  assert.throws(
    RangeError,
    () => instance.add({ hours: 1, minutes: -30 }, { overflow }),
    `mixed positive and negative values always throw (overflow = "${overflow}")`
  );
});
