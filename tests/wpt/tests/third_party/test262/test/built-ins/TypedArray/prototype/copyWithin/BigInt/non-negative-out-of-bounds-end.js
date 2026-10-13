// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Max value of end position is the this.length.
info: |
  22.2.3.5 %TypedArray%.prototype.copyWithin (target, start [ , end ] )

  %TypedArray%.prototype.copyWithin is a distinct function that implements the
  same algorithm as Array.prototype.copyWithin as defined in 22.1.3.3 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length" and the actual copying of values in step 12
  must be performed in a manner that preserves the bit-level encoding of the
  source data.

  ...
includes: [compareArray.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, 1, 6),
      [1n, 2n, 3n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, 1, 6) -> [1, 2, 3, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(0, 1, Infinity),
      [2n, 3n, 4n, 5n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(0, 1, Infinity) -> [2, 3, 4, 5, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n, 5n])).copyWithin(1, 3, 6),
      [0n, 3n, 4n, 5n, 4n, 5n]
    ),
    '[0, 1, 2, 3, 4, 5].copyWithin(1, 3, 6) -> [0, 3, 4, 5, 4, 5]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n])).copyWithin(1, 3, Infinity),
      [1n, 4n, 5n, 4n, 5n]
    ),
    '[1, 2, 3, 4, 5].copyWithin(1, 3, Infinity) -> [1, 4, 5, 4, 5]'
  );
}, null, null, ["immutable"]);
