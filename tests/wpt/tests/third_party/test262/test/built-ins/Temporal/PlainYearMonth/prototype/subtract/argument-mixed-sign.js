// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Positive and negative values in the temporalDurationLike argument are not acceptable
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 5);

["constrain", "reject"].forEach((overflow) => {
  assert.throws(
    RangeError,
    () => instance.subtract({ years: 1, months: -3 }, { overflow }),
    `mixed positive and negative values always throw (overflow = "${overflow}")`
  );
});
