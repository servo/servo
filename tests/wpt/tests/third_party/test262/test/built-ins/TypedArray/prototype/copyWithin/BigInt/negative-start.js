// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Set values with negative start argument.
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
  6. If relativeStart < 0, let from be max((len + relativeStart), 0); else let
  from be min(relativeStart, len).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, -1),
      [3n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, -1) -> [3, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n])).copyWithin(2, -2),
      [0n, 1n, 3n, 4n, 4n]
    ),
    '[0, 1, 2, 3, 4].copyWithin(2, -2) -> [0, 1, 3, 4, 4]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n])).copyWithin(1, -2),
      [0n, 3n, 4n, 3n, 4n]
    ),
    '[0, 1, 2, 3, 4].copyWithin(1, -2) -> [0, 3, 4, 3, 4]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(-1, -2),
      [0n, 1n, 2n, 2n]
    ),
    '[0, 1, 2, 3].copyWithin(-1, -2) -> [ 0, 1, 2, 2 ]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n])).copyWithin(-2, -3),
      [0n, 1n, 2n, 2n, 3n]
    ),
    '[0, 1, 2, 3, 4].copyWithin(-2, -3) -> [0, 1, 2, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n])).copyWithin(-5, -2),
      [3n, 4n, 2n, 3n, 4n]
    ),
    '[0, 1, 2, 3, 4].copyWithin(-5, -2) -> [3, 4, 2, 3, 4]'
  );
});
