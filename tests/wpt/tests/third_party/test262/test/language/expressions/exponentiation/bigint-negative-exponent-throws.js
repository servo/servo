// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: If the BigInt exponent is < 0, throw a RangeError exception
esid: sec-exp-operator-runtime-semantics-evaluation
info: |
  ExponentiationExpression: UpdateExpression ** ExponentiationExpression

  ...
  9. Return ? Type(base)::exponentiate(base, exponent).

  BigInt::exponentiate (base, exponent)

  1. If exponent < 0, throw a RangeError exception.
  ...
features: [BigInt, exponentiation]
---*/
assert.throws(RangeError, function() {
  1n ** -1n;
}, '1n ** -1n throws RangeError');

assert.throws(RangeError, function() {
  0n ** -1n;
}, '0n ** -1n throws RangeError');

assert.throws(RangeError, function() {
  (-1n) ** -1n;
}, '(-1n) ** -1n throws RangeError');

assert.throws(RangeError, function() {
  1n ** -100000000000000000n;
}, '1n ** -100000000000000000n throws RangeError');
