// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Left shift for non-primitive BigInt values
esid: sec-left-shift-operator-runtime-semantics-evaluation
info: |
  ShiftExpression : ShiftExpression << AdditiveExpression

  1. Let lref be the result of evaluating ShiftExpression.
  2. Let lval be ? GetValue(lref).
  3. Let rref be the result of evaluating AdditiveExpression.
  4. Let rval be ? GetValue(rref).
  5. Let lnum be ? ToNumeric(lval).
  6. Let rnum be ? ToNumeric(rval).
  7. If Type(lnum) does not equal Type(rnum), throw a TypeError exception.
  8. Let T be Type(lnum).
  9. Return T::leftShift(lnum, rnum).

features: [BigInt, Symbol.toPrimitive]
---*/
assert.sameValue(Object(0b101n) << 1n, 0b1010n, 'The result of (Object(0b101n) << 1n) is 0b1010n');

assert.sameValue(
  Object(0b101n) << Object(1n),
  0b1010n,
  'The result of (Object(0b101n) << Object(1n)) is 0b1010n'
);

function err() {
  throw new Test262Error();
}

assert.sameValue({
  [Symbol.toPrimitive]: function() {
    return 0b101n;
  },

  valueOf: err,
  toString: err
} << 1n, 0b1010n, 'The result of (({[Symbol.toPrimitive]: function() {return 0b101n;}, valueOf: err, toString: err}) << 1n) is 0b1010n');

assert.sameValue({
  valueOf: function() {
    return 0b101n;
  },

  toString: err
} << 1n, 0b1010n, 'The result of (({valueOf: function() {return 0b101n;}, toString: err}) << 1n) is 0b1010n');

assert.sameValue({
  toString: function() {
    return 0b101n;
  }
} << 1n, 0b1010n, 'The result of (({toString: function() {return 0b101n;}}) << 1n) is 0b1010n');

assert.sameValue(0b101n << {
  [Symbol.toPrimitive]: function() {
    return 1n;
  },

  valueOf: err,
  toString: err
}, 0b1010n, 'The result of (0b101n << {[Symbol.toPrimitive]: function() {return 1n;}, valueOf: err, toString: err}) is 0b1010n');

assert.sameValue(0b101n << {
  valueOf: function() {
    return 1n;
  },

  toString: err
}, 0b1010n, 'The result of (0b101n << {valueOf: function() {return 1n;}, toString: err}) is 0b1010n');

assert.sameValue(0b101n << {
  toString: function() {
    return 1n;
  }
}, 0b1010n, 'The result of (0b101n << {toString: function() {return 1n;}}) is 0b1010n');

assert.sameValue({
  valueOf: function() {
    return 0b101n;
  }
} << {
  valueOf: function() {
    return 1n;
  }
}, 0b1010n, 'The result of (({valueOf: function() {return 0b101n;}}) << {valueOf: function() {return 1n;}}) is 0b1010n');
