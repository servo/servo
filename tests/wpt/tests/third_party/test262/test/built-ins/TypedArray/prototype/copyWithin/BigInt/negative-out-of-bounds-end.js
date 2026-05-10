// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Set values with negative out of bounds end argument.
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
  7. If end is undefined, let relativeEnd be len; else let relativeEnd be ?
  ToInteger(end).
  8. If relativeEnd < 0, let final be max((len + relativeEnd), 0); else let
  final be min(relativeEnd, len).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, 1, -10),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, 1, -10) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(0, 1, -Infinity),
      [1n, 2n, 3n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(0, 1, -Infinity) -> [1, 2, 3, 4, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, -2, -10),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, -2, -10) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(0, -2, -Infinity),
      [1n, 2n, 3n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(0, -2, -Infinity) -> [1, 2, 3, 4, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, -9, -10),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, -9, -10) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(0, -9, -Infinity),
      [1n, 2n, 3n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(0, -9, -Infinity) -> [1, 2, 3, 4, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(-3, -2, -10),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(-3, -2, -10) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(-3, -2, -Infinity),
      [1n, 2n, 3n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(-3, -2, -Infinity) -> [1, 2, 3, 4, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(-7, -8, -9),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(-7, -8, -9) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(-7, -8, -Infinity),
      [1n, 2n, 3n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(-7, -8, -Infinity) -> [1, 2, 3, 4, 5]'
  );
});
