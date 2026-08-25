// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: round() throws without required smallestUnit parameter.
features: [Temporal]
---*/

const inst = new Temporal.Instant(0n);

assert.throws(RangeError, () => inst.round({}));
assert.throws(RangeError, () => inst.round({
  roundingIncrement: 1,
  roundingMode: "ceil"
}));
