// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-typedarray
description: >
  Passing a SharedArrayBuffer-backed TypedArray to a TypedArray constructor
  produces an ArrayBuffer-backed TypedArray.
includes: [testTypedArray.js]
features: [SharedArrayBuffer, TypedArray]
---*/

var sab = new SharedArrayBuffer(4);
var int_views = [Int8Array, Uint8Array, Int16Array, Uint16Array, Int32Array, Uint32Array];

testWithTypedArrayConstructors(function(View1, makeCtorArg) {
  var ta1 = new View1(sab);
  testWithTypedArrayConstructors(function(View2, makeCtorArg) {
    var ta2 = new View2(ta1);
    assert.sameValue(ta2.buffer.constructor, ArrayBuffer,
                     "TypedArray of SharedArrayBuffer-backed TypedArray is ArrayBuffer-backed");
  }, int_views);
}, int_views);
