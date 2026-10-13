// Copyright (C) 2016 The V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.isview
description: >
  `isView` can be invoked as a function
info: |
  24.1.3.1 ArrayBuffer.isView ( arg )

  1. If Type(arg) is not Object, return false.
  2. If arg has a [[ViewedArrayBuffer]] internal slot, return true.
  3. Return false.
features: [TypedArray, DataView]
includes: [testTypedArray.js]
---*/

var isView = ArrayBuffer.isView;

testWithAllTypedArrayConstructors(function(ctor, makeCtorArg) {
  var sample = new ctor(makeCtorArg(0));
  assert.sameValue(isView(sample), true, "instance of TypedArray");
});

var dv = new DataView(new ArrayBuffer(1), 0, 0);
assert.sameValue(isView(dv), true, "instance of DataView");

assert.sameValue(isView(), false, "undefined arg");
assert.sameValue(isView({}), false, "ordinary object");
