// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-length
description: >
  Throws a RangeError if ToInteger(length) is a negative value
info: |
  22.2.4.2 TypedArray ( length )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is not Object.

  ...
  3. Let elementLength be ? ToIndex(length).
  ...

  7.1.17 ToIndex ( value )

  1. If value is undefined, then
    ...
  2. Else,
    a. Let integerIndex be ? ToInteger(value).
    b. If integerIndex < 0, throw a RangeError exception.
    ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.throws(RangeError, function() {
    new TA(-1);
  });

  assert.throws(RangeError, function() {
    new TA(-Infinity);
  });
}, null, ["passthrough"]);
