// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Set values with out of bounds negative target argument.
info: |
  22.2.3.5 %TypedArray%.prototype.copyWithin (target, start [ , end ] )

  %TypedArray%.prototype.copyWithin is a distinct function that implements the
  same algorithm as Array.prototype.copyWithin as defined in 22.1.3.3 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length" and the actual copying of values in step 12
  must be performed in a manner that preserves the bit-level encoding of the
  source data.

  ...

  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  4. If relativeTarget < 0, let to be max((len + relativeTarget), 0); else let
  to be min(relativeTarget, len).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(-10, 0),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(-10, 0) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(-Infinity, 0),
      [1n, 2n, 3n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(-Infinity, 0) -> [1, 2, 3, 4, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n])).copyWithin(-10, 2),
      [2n, 3n, 4n, 3n, 4n]
    ),
    '[0, 1, 2, 3, 4].copyWithin(-10, 2) -> [2, 3, 4, 3, 4]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(-Infinity, 2),
      [3n, 4n, 5n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(-Infinity, 2) -> [3, 4, 5, 4, 5]'
  );
});
