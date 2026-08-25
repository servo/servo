// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Bitwise NOT for BigInt object wrappers
esid: sec-bitwise-not-operator-runtime-semantics-evaluation
info: |
  Runtime Semantics: Evaluation
  UnaryExpression : ~ UnaryExpression

  1. Let expr be the result of evaluating UnaryExpression.
  2. Let oldValue be ? ToNumeric(? GetValue(expr)).
  3. Let T be Type(oldValue).
  4. Return ? T::bitwiseNOT(oldValue).

features: [BigInt, Symbol.toPrimitive]
---*/
assert.sameValue(~Object(1n), -2n, 'The value of ~Object(1n) is -2n');

function err() {
  throw new Test262Error();
}

assert.sameValue(~{
  [Symbol.toPrimitive]: function() {
    return 1n;
  },

  valueOf: err,
  toString: err
}, -2n, 'The value of ~{[Symbol.toPrimitive]: function() {return 1n;}, valueOf: err, toString: err} is -2n');

assert.sameValue(~{
  valueOf: function() {
    return 1n;
  },

  toString: err
}, -2n, 'The value of ~{valueOf: function() {return 1n;}, toString: err} is -2n');

assert.sameValue(~{
  toString: function() {
    return 1n;
  }
}, -2n, 'The value of ~{toString: function() {return 1n;}} is -2n');
