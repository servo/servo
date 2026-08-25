// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Bitwise OR for BigInt non-primitive values
esid: sec-binary-bitwise-operators-runtime-semantics-evaluation
info: |
  5. Let lnum be ? ToNumeric(lval).
  6. Let rnum be ? ToNumeric(rval).
  ...
  8. Let T be Type(lnum).
  ...
  10. If @ is |, return T::bitwiseOR(lnum, rnum).

features: [BigInt]
---*/
assert.sameValue(
  Object(0b101n) | 0b011n,
  0b111n,
  'The result of (Object(0b101n) | 0b011n) is 0b111n'
);

assert.sameValue(
  0b011n | Object(0b101n),
  0b111n,
  'The result of (0b011n | Object(0b101n)) is 0b111n'
);

assert.sameValue(
  Object(0b101n) | Object(0b011n),
  0b111n,
  'The result of (Object(0b101n) | Object(0b011n)) is 0b111n'
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
} | 0b011n, 0b111n, 'The result of (({[Symbol.toPrimitive]: function() {return 0b101n;}, valueOf: err, toString: err}) | 0b011n) is 0b111n');

assert.sameValue(0b011n | {
  [Symbol.toPrimitive]: function() {
    return 0b101n;
  },

  valueOf: err,
  toString: err
}, 0b111n, 'The result of (0b011n | {[Symbol.toPrimitive]: function() {return 0b101n;}, valueOf: err, toString: err}) is 0b111n');

assert.sameValue({
  valueOf: function() {
    return 0b101n;
  },

  toString: err
} | 0b011n, 0b111n, 'The result of (({valueOf: function() {return 0b101n;}, toString: err}) | 0b011n) is 0b111n');

assert.sameValue(0b011n | {
  valueOf: function() {
    return 0b101n;
  },

  toString: err
}, 0b111n, 'The result of (0b011n | {valueOf: function() {return 0b101n;}, toString: err}) is 0b111n');

assert.sameValue({
  toString: function() {
    return 0b101n;
  }
} | 0b011n, 0b111n, 'The result of (({toString: function() {return 0b101n;}}) | 0b011n) is 0b111n');

assert.sameValue(0b011n | {
  toString: function() {
    return 0b101n;
  }
}, 0b111n, 'The result of (0b011n | {toString: function() {return 0b101n;}}) is 0b111n');
