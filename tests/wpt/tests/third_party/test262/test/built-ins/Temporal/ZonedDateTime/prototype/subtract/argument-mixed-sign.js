// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Positive and negative values in the temporalDurationLike argument are not acceptable
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

["constrain", "reject"].forEach((overflow) => {
  assert.throws(
    RangeError,
    () => instance.subtract({ hours: 1, minutes: -30 }, { overflow }),
    `mixed positive and negative values always throw (overflow = "${overflow}")`
  );
});
