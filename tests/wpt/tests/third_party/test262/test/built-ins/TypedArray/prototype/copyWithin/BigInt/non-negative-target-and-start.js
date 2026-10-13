// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Copy values with non-negative target and start positions.
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
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n, 6n])).copyWithin(0, 0),
      [1n, 2n, 3n, 4n, 5n, 6n]
    )
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n, 6n])).copyWithin(0, 2),
      [3n, 4n, 5n, 6n, 5n, 6n]
    )
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1n, 2n, 3n, 4n, 5n, 6n])).copyWithin(3, 0),
      [1n, 2n, 3n, 1n, 2n, 3n]
    )
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n, 5n])).copyWithin(1, 4),
      [0n, 4n, 5n, 3n, 4n, 5n]
    )
  );
}, null, null, ["immutable"]);
