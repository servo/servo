// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Negative fractional index which is truncated to zero.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  4. Let relativeIndex be ? ToIntegerOrInfinity(index).
  ...

  ToIntegerOrInfinity ( argument )

  1. Let number be ? ToNumber(argument).
  2. If number is one of NaN, +0𝔽, or -0𝔽, return 0.
  3. If number is +∞𝔽, return +∞.
  4. If number is -∞𝔽, return -∞.
  5. Return truncate(ℝ(number)).
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta = new TA(makeCtorArg(1));
  var result = ta.with(-0.5, 123);
  assert.sameValue(result[0], 123);
});
