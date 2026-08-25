// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict equality comparison of BigInt and non-finite Number values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(0n === Infinity, false, 'The result of (0n === Infinity) is false');
assert.sameValue(Infinity === 0n, false, 'The result of (Infinity === 0n) is false');
assert.sameValue(1n === Infinity, false, 'The result of (1n === Infinity) is false');
assert.sameValue(Infinity === 1n, false, 'The result of (Infinity === 1n) is false');
assert.sameValue(-1n === Infinity, false, 'The result of (-1n === Infinity) is false');
assert.sameValue(Infinity === -1n, false, 'The result of (Infinity === -1n) is false');
assert.sameValue(0n === -Infinity, false, 'The result of (0n === -Infinity) is false');
assert.sameValue(-Infinity === 0n, false, 'The result of (-Infinity === 0n) is false');
assert.sameValue(1n === -Infinity, false, 'The result of (1n === -Infinity) is false');
assert.sameValue(-Infinity === 1n, false, 'The result of (-Infinity === 1n) is false');
assert.sameValue(-1n === -Infinity, false, 'The result of (-1n === -Infinity) is false');
assert.sameValue(-Infinity === -1n, false, 'The result of (-Infinity === -1n) is false');
assert.sameValue(0n === NaN, false, 'The result of (0n === NaN) is false');
assert.sameValue(NaN === 0n, false, 'The result of (NaN === 0n) is false');
assert.sameValue(1n === NaN, false, 'The result of (1n === NaN) is false');
assert.sameValue(NaN === 1n, false, 'The result of (NaN === 1n) is false');
assert.sameValue(-1n === NaN, false, 'The result of (-1n === NaN) is false');
assert.sameValue(NaN === -1n, false, 'The result of (NaN === -1n) is false');
