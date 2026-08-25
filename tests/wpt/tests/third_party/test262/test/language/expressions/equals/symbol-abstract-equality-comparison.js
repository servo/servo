// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 7.2.12
description: >
    Abstract Equality Comparison: Symbol
features: [Symbol]
---*/
var symA = Symbol('66');
var symB = Symbol('66');

assert.sameValue(symA == symA, true, "The result of `symA == symA` is `true`");
assert.sameValue(symA == symA.valueOf(), true, "The result of `symA == symA.valueOf()` is `true`");
assert.sameValue(symA.valueOf() == symA, true, "The result of `symA.valueOf() == symA` is `true`");

assert.sameValue(symB == symB, true, "The result of `symB == symB` is `true`");
assert.sameValue(symB == symB.valueOf(), true, "The result of `symB == symB.valueOf()` is `true`");
assert.sameValue(symB.valueOf() == symB, true, "The result of `symB.valueOf() == symB` is `true`");

assert.sameValue(symA == symB, false, "The result of `symA == symB` is `false`");

