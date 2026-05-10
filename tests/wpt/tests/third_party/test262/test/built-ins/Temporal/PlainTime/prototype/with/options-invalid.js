// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: TypeError thrown when a primitive is passed as the options argument
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(12);
for (const badOptions of [null, true, "hello", Symbol("foo"), 1, 1n]) {
  assert.throws(TypeError, () => plainTime.with({ hour: 3 }, badOptions));
}
