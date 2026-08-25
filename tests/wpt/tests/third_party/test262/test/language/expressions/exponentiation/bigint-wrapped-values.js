// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: exponentiation operator ToNumeric with BigInt operands
esid: sec-exp-operator-runtime-semantics-evaluation
features: [BigInt, Symbol.toPrimitive, computed-property-names, exponentiation]
---*/
assert.sameValue(Object(2n) ** 1n, 2n, 'The result of (Object(2n) ** 1n) is 2n');
assert.sameValue(1n ** Object(2n), 1n, 'The result of (1n ** Object(2n)) is 1n');

assert.sameValue({
  [Symbol.toPrimitive]: function() {
    return 2n;
  }
} ** 1n, 2n, 'The result of (({[Symbol.toPrimitive]: function() {return 2n;}}) ** 1n) is 2n');

assert.sameValue(1n ** {
  [Symbol.toPrimitive]: function() {
    return 2n;
  }
}, 1n, 'The result of (1n ** {[Symbol.toPrimitive]: function() {return 2n;}}) is 1n');

assert.sameValue({
  valueOf: function() {
    return 2n;
  }
} ** 1n, 2n, 'The result of (({valueOf: function() {return 2n;}}) ** 1n) is 2n');

assert.sameValue(1n ** {
  valueOf: function() {
    return 2n;
  }
}, 1n, 'The result of (1n ** {valueOf: function() {return 2n;}}) is 1n');

assert.sameValue({
  toString: function() {
    return 2n;
  }
} ** 1n, 2n, 'The result of (({toString: function() {return 2n;}}) ** 1n) is 2n');

assert.sameValue(1n ** {
  toString: function() {
    return 2n;
  }
}, 1n, 'The result of (1n ** {toString: function() {return 2n;}}) is 1n');
