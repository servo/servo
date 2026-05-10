// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: month and monthCode must agree.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

// month and monthCode must agree
assert.throws(RangeError, () => zdt.with({
  month: 5,
  monthCode: "M06"
}));
