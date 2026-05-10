// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Non-strict inequality comparison of BigInt and miscellaneous primitive values
esid: sec-equality-operators-runtime-semantics-evaluation
info: |
  EqualityExpression : EqualityExpression != RelationalExpression
    ...
    5. Return the result of performing Abstract Equality Comparison rval == lval.
    6. If r is true, return false. Otherwise, return true.

features: [BigInt, Symbol]
---*/
assert.sameValue(0n != undefined, true, 'The result of (0n != undefined) is true');
assert.sameValue(undefined != 0n, true, 'The result of (undefined != 0n) is true');
assert.sameValue(1n != undefined, true, 'The result of (1n != undefined) is true');
assert.sameValue(undefined != 1n, true, 'The result of (undefined != 1n) is true');
assert.sameValue(0n != null, true, 'The result of (0n != null) is true');
assert.sameValue(null != 0n, true, 'The result of (null != 0n) is true');
assert.sameValue(1n != null, true, 'The result of (1n != null) is true');
assert.sameValue(null != 1n, true, 'The result of (null != 1n) is true');
assert.sameValue(0n != Symbol('1'), true, 'The result of (0n != Symbol("1")) is true');
assert.sameValue(Symbol('1') != 0n, true, 'The result of (Symbol("1") != 0n) is true');
assert.sameValue(1n != Symbol('1'), true, 'The result of (1n != Symbol("1")) is true');
assert.sameValue(Symbol('1') != 1n, true, 'The result of (Symbol("1") != 1n) is true');
