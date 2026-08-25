// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: bitwise-xor operator ToNumeric with BigInt operands
esid: sec-binary-bitwise-operators-runtime-semantics-evaluation
features: [BigInt, Symbol.toPrimitive, computed-property-names]
---*/
assert.sameValue(Object(2n) ^ 3n, 1n, 'The result of (Object(2n) ^ 3n) is 1n');
assert.sameValue(3n ^ Object(2n), 1n, 'The result of (3n ^ Object(2n)) is 1n');

assert.sameValue({
  [Symbol.toPrimitive]: function() {
    return 2n;
  }
} ^ 3n, 1n, 'The result of (({[Symbol.toPrimitive]: function() {return 2n;}}) ^ 3n) is 1n');

assert.sameValue(3n ^ {
  [Symbol.toPrimitive]: function() {
    return 2n;
  }
}, 1n, 'The result of (3n ^ {[Symbol.toPrimitive]: function() {return 2n;}}) is 1n');

assert.sameValue({
  valueOf: function() {
    return 2n;
  }
} ^ 3n, 1n, 'The result of (({valueOf: function() {return 2n;}}) ^ 3n) is 1n');

assert.sameValue(3n ^ {
  valueOf: function() {
    return 2n;
  }
}, 1n, 'The result of (3n ^ {valueOf: function() {return 2n;}}) is 1n');

assert.sameValue({
  toString: function() {
    return 2n;
  }
} ^ 3n, 1n, 'The result of (({toString: function() {return 2n;}}) ^ 3n) is 1n');

assert.sameValue(3n ^ {
  toString: function() {
    return 2n;
  }
}, 1n, 'The result of (3n ^ {toString: function() {return 2n;}}) is 1n');
