// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: TypeError thrown when options argument is a primitive, before early return
features: [BigInt, Symbol, Temporal]
---*/

const badOptions = [
  null,
  true,
  "some string",
  Symbol(),
  1,
  2n,
];

for (const value of badOptions) {
  assert.throws(TypeError, () => Temporal.Duration.compare({ hours: 1 }, { hours: 1 }, value),
    `TypeError on wrong options type ${typeof value}`);
};
