// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Unary minus for BigInt object wrappers
esid: sec-unary-minus-operator-runtime-semantics-evaluation
info: |
  Runtime Semantics: Evaluation
  UnaryExpression : - UnaryExpression

  1. Let expr be the result of evaluating UnaryExpression.
  2. Let oldValue be ? ToNumeric(? GetValue(expr)).
  3. Let T be Type(oldValue).
  4. Return ? T::unaryMinus(oldValue).

features: [BigInt, Symbol.toPrimitive]
---*/
assert.sameValue(-Object(1n), -1n, 'The value of -Object(1n) is -1n');
assert.notSameValue(-Object(1n), 1n, 'The value of -Object(1n) is not 1n');
assert.notSameValue(-Object(1n), Object(-1n), 'The value of -Object(1n) is not Object(-1n)');
assert.sameValue(-Object(-1n), 1n, 'The value of -Object(-1n) is 1n');
assert.notSameValue(-Object(-1n), -1n, 'The value of -Object(-1n) is not -1n');
assert.notSameValue(-Object(-1n), Object(1n), 'The value of -Object(-1n) is not Object(1n)');

assert.sameValue(-{
  [Symbol.toPrimitive]: function() {
    return 1n;
  },

  valueOf: function() {
    throw new Test262Error();
  },

  toString: function() {
    throw new Test262Error();
  }
}, -1n, 'The value of -{[Symbol.toPrimitive]: function() {return 1n;}, valueOf: function() {throw new Test262Error();}, toString: function() {throw new Test262Error();}} is -1n');

assert.sameValue(-{
  valueOf: function() {
    return 1n;
  },

  toString: function() {
    throw new Test262Error();
  }
}, -1n, 'The value of -{valueOf: function() {return 1n;}, toString: function() {throw new Test262Error();}} is -1n');

assert.sameValue(-{
  toString: function() {
    return 1n;
  }
}, -1n, 'The value of -{toString: function() {return 1n;}} is -1n');
