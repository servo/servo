// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  If typedArray constructor argument is a Big(U)Int, succeed in set
info: |
  %TypedArray%.prototype.set( typedArray [ , offset ] )
  Sets multiple values in this TypedArray, reading the values from the
  typedArray argument object. The optional offset value indicates the first
  element index in this TypedArray where values are written. If omitted, it
  is assumed to be 0.
  ...
  23. If one of srcType and targetType contains the substring "Big" and the
      other does not, throw a TypeError exception.
  ...

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var srcTypedArray;
var targetTypedArray;
var testValue = 42n;

testWithBigIntTypedArrayConstructors(function(BTA1, makeCtorArg) {

  srcTypedArray = new BTA1(makeCtorArg([testValue]));

  testWithBigIntTypedArrayConstructors(function(BTA2, makeCtorArg) {

    targetTypedArray = new BTA2(1);
    targetTypedArray.set(srcTypedArray);
    assert.sameValue(targetTypedArray[0], testValue,
                     "Setting BigInt TypedArray with BigInt TypedArray should succeed.")
  });
}, null, null, ["immutable"]);

