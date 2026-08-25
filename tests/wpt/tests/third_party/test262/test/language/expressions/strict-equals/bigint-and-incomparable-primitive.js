// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict equality comparison of BigInt and miscellaneous primitive values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt, Symbol]
---*/
assert.sameValue(0n === undefined, false, 'The result of (0n === undefined) is false');
assert.sameValue(undefined === 0n, false, 'The result of (undefined === 0n) is false');
assert.sameValue(1n === undefined, false, 'The result of (1n === undefined) is false');
assert.sameValue(undefined === 1n, false, 'The result of (undefined === 1n) is false');
assert.sameValue(0n === null, false, 'The result of (0n === null) is false');
assert.sameValue(null === 0n, false, 'The result of (null === 0n) is false');
assert.sameValue(1n === null, false, 'The result of (1n === null) is false');
assert.sameValue(null === 1n, false, 'The result of (null === 1n) is false');
assert.sameValue(0n === Symbol('1'), false, 'The result of (0n === Symbol("1")) is false');
assert.sameValue(Symbol('1') === 0n, false, 'The result of (Symbol("1") === 0n) is false');
assert.sameValue(1n === Symbol('1'), false, 'The result of (1n === Symbol("1")) is false');
assert.sameValue(Symbol('1') === 1n, false, 'The result of (Symbol("1") === 1n) is false');
