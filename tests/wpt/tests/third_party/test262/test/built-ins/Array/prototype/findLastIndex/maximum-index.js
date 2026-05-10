// Copyright (C) 2023 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  Support the maximum valid integer index.
info: |
  Array.prototype.findLastIndex ( predicate [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).

  LengthOfArrayLike ( obj )

  1. Return ℝ(? ToLength(? Get(obj, "length"))).

  ToLength ( argument )

  1. Let len be ? ToIntegerOrInfinity(argument).
  2. If len ≤ 0, return +0𝔽.
  3. Return 𝔽(min(len, 2**53 - 1)).
features: [array-find-from-last]
---*/

var tooBigLength = Number.MAX_VALUE;
var maxExpectedIndex = 9007199254740990;
var arrayLike = { length: tooBigLength };
var calledWithIndex = [];

Array.prototype.findLastIndex.call(arrayLike, function(_value, index) {
  calledWithIndex.push(index);
  return true;
});

assert.sameValue(calledWithIndex.length, 1, 'predicate invoked once');
assert.sameValue(calledWithIndex[0], maxExpectedIndex);
