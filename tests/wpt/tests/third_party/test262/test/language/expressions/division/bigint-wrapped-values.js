// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: division operator ToNumeric with BigInt operands
esid: sec-multiplicative-operators-runtime-semantics-evaluation
features: [BigInt, Symbol.toPrimitive, computed-property-names]
---*/
assert.sameValue(Object(2n) / 2n, 1n, 'The result of (Object(2n) / 2n) is 1n');
assert.sameValue(2n / Object(2n), 1n, 'The result of (2n / Object(2n)) is 1n');

assert.sameValue({
  [Symbol.toPrimitive]: function() {
    return 2n;
  }
} / 2n, 1n, 'The result of (({[Symbol.toPrimitive]: function() {return 2n;}}) / 2n) is 1n');

assert.sameValue(2n / {
  [Symbol.toPrimitive]: function() {
    return 2n;
  }
}, 1n, 'The result of (2n / {[Symbol.toPrimitive]: function() {return 2n;}}) is 1n');

assert.sameValue({
  valueOf: function() {
    return 2n;
  }
} / 2n, 1n, 'The result of (({valueOf: function() {return 2n;}}) / 2n) is 1n');

assert.sameValue(2n / {
  valueOf: function() {
    return 2n;
  }
}, 1n, 'The result of (2n / {valueOf: function() {return 2n;}}) is 1n');

assert.sameValue({
  toString: function() {
    return 2n;
  }
} / 2n, 1n, 'The result of (({toString: function() {return 2n;}}) / 2n) is 1n');

assert.sameValue(2n / {
  toString: function() {
    return 2n;
  }
}, 1n, 'The result of (2n / {toString: function() {return 2n;}}) is 1n');
