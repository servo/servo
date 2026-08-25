// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict inequality comparison of BigInt and non-finite Number values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(0n !== Infinity, true, 'The result of (0n !== Infinity) is true');
assert.sameValue(Infinity !== 0n, true, 'The result of (Infinity !== 0n) is true');
assert.sameValue(1n !== Infinity, true, 'The result of (1n !== Infinity) is true');
assert.sameValue(Infinity !== 1n, true, 'The result of (Infinity !== 1n) is true');
assert.sameValue(-1n !== Infinity, true, 'The result of (-1n !== Infinity) is true');
assert.sameValue(Infinity !== -1n, true, 'The result of (Infinity !== -1n) is true');
assert.sameValue(0n !== -Infinity, true, 'The result of (0n !== -Infinity) is true');
assert.sameValue(-Infinity !== 0n, true, 'The result of (-Infinity !== 0n) is true');
assert.sameValue(1n !== -Infinity, true, 'The result of (1n !== -Infinity) is true');
assert.sameValue(-Infinity !== 1n, true, 'The result of (-Infinity !== 1n) is true');
assert.sameValue(-1n !== -Infinity, true, 'The result of (-1n !== -Infinity) is true');
assert.sameValue(-Infinity !== -1n, true, 'The result of (-Infinity !== -1n) is true');
assert.sameValue(0n !== NaN, true, 'The result of (0n !== NaN) is true');
assert.sameValue(NaN !== 0n, true, 'The result of (NaN !== 0n) is true');
assert.sameValue(1n !== NaN, true, 'The result of (1n !== NaN) is true');
assert.sameValue(NaN !== 1n, true, 'The result of (NaN !== 1n) is true');
assert.sameValue(-1n !== NaN, true, 'The result of (-1n !== NaN) is true');
assert.sameValue(NaN !== -1n, true, 'The result of (NaN !== -1n) is true');
