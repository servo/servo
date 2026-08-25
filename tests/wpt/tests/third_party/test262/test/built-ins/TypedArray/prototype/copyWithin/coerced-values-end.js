// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  end argument is coerced to an integer values.
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
  ...
includes: [compareArray.js, testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1, 0, null),
      [0, 1, 2, 3]
    ),
    'null value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1, 0, NaN),
      [0, 1, 2, 3]
    ),
    'NaN value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1, 0, false),
      [0, 1, 2, 3]
    ),
    'false value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1, 0, true),
      [0, 0, 2, 3]
    ),
    'true value coerced to 1'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1, 0, '-2'),
      [0, 0, 1, 3]
    ),
    'string "-2" value coerced to integer -2'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1, 0, -2.5),
      [0, 0, 1, 3]
    ),
    'float -2.5 value coerced to integer -2'
  );
}, null, null, ["immutable"]);
