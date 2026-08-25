// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt modulo 0 throws a range error
esid: sec-multiplicative-operators-runtime-semantics-evaluation
info: |
  Runtime Semantics: Evaluation

  MultiplicativeExpression: MultiplicativeExpression MultiplicativeOperator ExponentiationExpression

  ...
  12. Otherwise, MultiplicativeOperator is %; return T::remainder(lnum, rnum).
  ...

  BigInt::remainder (x, y)

  1. If y is 0n, throw a RangeError exception.
  2. Return the BigInt representing x modulo y.
features: [BigInt]
---*/
assert.throws(RangeError, function() {
  1n % 0n;
}, '1n % 0n throws RangeError');

assert.throws(RangeError, function() {
  10n % 0n;
}, '10n % 0n throws RangeError');

assert.throws(RangeError, function() {
  0n % 0n;
}, '0n % 0n throws RangeError');

assert.throws(RangeError, function() {
  1000000000000000000n % 0n;
}, '1000000000000000000n % 0n throws RangeError');
