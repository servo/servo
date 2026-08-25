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
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(undefined, 1),
      [1n, 2n, 3n, 3n]
    ),
    'undefined value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(false, 1),
      [1n, 2n, 3n, 3n]
    ),
    'false value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(NaN, 1),
      [1n, 2n, 3n, 3n]
    ),
    'NaN value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(null, 1),
      [1n, 2n, 3n, 3n]
    ),
    'null value coerced to 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(true, 0),
      [0n, 0n, 1n, 2n]
    ),
    'true value coerced to 1'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin('1', 0),
      [0n, 0n, 1n, 2n]
    ),
    'string "1" value coerced to 1'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(0.5, 1),
      [1n, 2n, 3n, 3n]
    ),
    '0.5 float value coerced to integer 0'
  );

  assert(
    compareArray(
      new TA(makeCtorArg([0n, 1n, 2n, 3n])).copyWithin(1.5, 0),
      [0n, 0n, 1n, 2n]
    ),
    '1.5 float value coerced to integer 1'
  );
}, null, null, ["immutable"]);
