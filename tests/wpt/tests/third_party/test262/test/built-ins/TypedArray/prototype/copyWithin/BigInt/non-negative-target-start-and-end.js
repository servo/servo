// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  Copy values with non-negative target, start and end positions.
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
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, 0, 0),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, 0, 0) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, 0, 2),
      [0n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, 0, 2) -> [0, 1, 2, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0, 1, 2),
      [1n, 1n, 2n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(0, 1, 2) -> [1, 1, 2, 3]'
  );

  /*
   * 10. If from<to and to<from+count, then
   *   a. Let direction be - 1.
   *   b. Let from be from + count - 1.
   *   c. Let to be to + count - 1.
   *
   *  0 < 1, 1 < 0 + 2
   *  direction = -1
   *  from = 0 + 2 - 1
   *  to = 1 + 2 - 1
   */
  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(1, 0, 2),
      [0n, 0n, 1n, 3n]
    ),
    '[0, 1, 2, 3].copyWithin(1, 0, 2) -> [0, 0, 1, 3]'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n, 4n, 5n])).copyWithin(1, 3, 5),
      [0n, 3n, 4n, 3n, 4n, 5n]
    ),
    '[0, 1, 2, 3, 4, 5].copyWithin(1, 3, 5) -> [0, 3, 4, 3, 4, 5]'
  );
});
