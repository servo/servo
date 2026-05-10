// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict inequality comparison of BigInt and Boolean values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(-1n !== false, true, 'The result of (-1n !== false) is true');
assert.sameValue(false !== -1n, true, 'The result of (false !== -1n) is true');
assert.sameValue(-1n !== true, true, 'The result of (-1n !== true) is true');
assert.sameValue(true !== -1n, true, 'The result of (true !== -1n) is true');
assert.sameValue(0n !== false, true, 'The result of (0n !== false) is true');
assert.sameValue(false !== 0n, true, 'The result of (false !== 0n) is true');
assert.sameValue(0n !== true, true, 'The result of (0n !== true) is true');
assert.sameValue(true !== 0n, true, 'The result of (true !== 0n) is true');
assert.sameValue(1n !== false, true, 'The result of (1n !== false) is true');
assert.sameValue(false !== 1n, true, 'The result of (false !== 1n) is true');
assert.sameValue(1n !== true, true, 'The result of (1n !== true) is true');
assert.sameValue(true !== 1n, true, 'The result of (true !== 1n) is true');
assert.sameValue(2n !== false, true, 'The result of (2n !== false) is true');
assert.sameValue(false !== 2n, true, 'The result of (false !== 2n) is true');
assert.sameValue(2n !== true, true, 'The result of (2n !== true) is true');
assert.sameValue(true !== 2n, true, 'The result of (true !== 2n) is true');
