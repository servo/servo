// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  `from` is %TypedArray%.from
info: |
  22.2.1 The %TypedArray% Intrinsic Object

  The %TypedArray% intrinsic object is a constructor function object that all of
  the TypedArray constructor object inherit from.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.sameValue(
    TA.from, TypedArray.from,
    "method is inherited %TypedArray%.from"
  );
  assert.sameValue(
    TA.hasOwnProperty("from"), false,
    "constructor does not define an own property named 'from'"
  );
}, null, ["passthrough"]);
