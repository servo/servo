// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict equality comparison of BigInt and Boolean values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(-1n === false, false, 'The result of (-1n === false) is false');
assert.sameValue(false === -1n, false, 'The result of (false === -1n) is false');
assert.sameValue(-1n === true, false, 'The result of (-1n === true) is false');
assert.sameValue(true === -1n, false, 'The result of (true === -1n) is false');
assert.sameValue(0n === false, false, 'The result of (0n === false) is false');
assert.sameValue(false === 0n, false, 'The result of (false === 0n) is false');
assert.sameValue(0n === true, false, 'The result of (0n === true) is false');
assert.sameValue(true === 0n, false, 'The result of (true === 0n) is false');
assert.sameValue(1n === false, false, 'The result of (1n === false) is false');
assert.sameValue(false === 1n, false, 'The result of (false === 1n) is false');
assert.sameValue(1n === true, false, 'The result of (1n === true) is false');
assert.sameValue(true === 1n, false, 'The result of (true === 1n) is false');
assert.sameValue(2n === false, false, 'The result of (2n === false) is false');
assert.sameValue(false === 2n, false, 'The result of (false === 2n) is false');
assert.sameValue(2n === true, false, 'The result of (2n === true) is false');
assert.sameValue(true === 2n, false, 'The result of (true === 2n) is false');
