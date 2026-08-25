// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The Unary + Operator throws a TypeError on BigInt numbers
esid: sec-unary-plus-operator-runtime-semantics-evaluation
info: |
  UnaryExpression: + UnaryExpression

  1. Let expr be the result of evaluating UnaryExpression.
  2. Return ? ToNumber(? GetValue(expr)).

  ToNumber ( argument )

  BigInt: Throw a TypeError exception
features: [BigInt]
---*/
assert.throws(TypeError, function() {
  +0n;
}, '+0n throws TypeError');

assert.throws(TypeError, function() {
  +1n;
}, '+1n throws TypeError');

assert.throws(TypeError, function() {
  +-1n;
}, '+-1n throws TypeError');

assert.throws(TypeError, function() {
  +1000000000000000n;
}, '+1000000000000000n throws TypeError');
