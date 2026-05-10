// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: TypeError thrown when a primitive is passed as the options argument
features: [Temporal]
---*/

const values = [null, true, "hello", Symbol("foo"), 1, 1n];
const time = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
const one = new Temporal.PlainTime(16, 23, 30, 123, 456, 789);

for (const badOptions of values) {
  assert.throws(TypeError, () => time.until(one, badOptions));
}
