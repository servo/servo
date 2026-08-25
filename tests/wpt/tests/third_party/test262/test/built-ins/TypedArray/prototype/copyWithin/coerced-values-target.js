// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  target argument is coerced to an integer value.
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
  3. Let relativeTarget be ? ToInteger(target).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(undefined, 1),
      [1, 2, 3, 3]
    ),
    'undefined value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(false, 1),
      [1, 2, 3, 3]
    ),
    'false value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(NaN, 1),
      [1, 2, 3, 3]
    ),
    'NaN value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(null, 1),
      [1, 2, 3, 3]
    ),
    'null value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(true, 0),
      [0, 0, 1, 2]
    ),
    'true value coerced to 1'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin('1', 0),
      [0, 0, 1, 2]
    ),
    'string "1" value coerced to 1'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(0.5, 1),
      [1, 2, 3, 3]
    ),
    '0.5 float value coerced to integer 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin(1.5, 0),
      [0, 0, 1, 2]
    ),
    '1.5 float value coerced to integer 1'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0, 1, 2, 3])).copyWithin({}, 1),
      [1, 2, 3, 3]
    ),
    'object value coerced to integer 0'
  );
}, null, null, ["immutable"]);
