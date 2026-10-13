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
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([1, 2, 3, 4, 5, 6])).copyWithin(0, 0),
      [1, 2, 3, 4, 5, 6]
    )
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1, 2, 3, 4, 5, 6])).copyWithin(0, 2),
      [3, 4, 5, 6, 5, 6]
    )
  );

  assert(
    compareArray(
      new TA(makeCtorArg([1, 2, 3, 4, 5, 6])).copyWithin(3, 0),
      [1, 2, 3, 1, 2, 3]
    )
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3, 4, 5])).copyWithin(1, 4),
      [0, 4, 5, 3, 4, 5]
    )
  );
}, null, null, ["immutable"]);
