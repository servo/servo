// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: TypeError thrown when a primitive is passed as the options argument
features: [Temporal]
---*/

const values = [null, true, "hello", Symbol("foo"), 1, 1n];

for (const badOptions of values) {
  assert.throws(TypeError, () => Temporal.PlainTime.from({ hours: 12 }, badOptions));
}
