// Copyright 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.pause
description: Atomics.pause returns undefined
features: [Atomics.pause]
---*/

assert.sameValue(Atomics.pause(), undefined,
                 'Atomics.pause returns undefined');

const values = [
  undefined,
  42,
  0,
  -0,
  Number.MAX_SAFE_INTEGER
];

for (const v of values) {
  assert.sameValue(Atomics.pause(v), undefined,
                   'Atomics.pause returns undefined');
}
