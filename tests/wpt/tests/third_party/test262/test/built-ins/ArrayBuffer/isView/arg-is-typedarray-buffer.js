// Copyright (C) 2016 The V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.isview
description: >
  Return false from TypedArray's instance `.buffer`
info: |
  24.1.3.1 ArrayBuffer.isView ( arg )

  1. If Type(arg) is not Object, return false.
  2. If arg has a [[ViewedArrayBuffer]] internal slot, return true.
  3. Return false.
features: [TypedArray]
includes: [testTypedArray.js]
---*/

testWithTypedArrayConstructors(function(ctor) {
  var sample = new ctor().buffer;

  assert.sameValue(ArrayBuffer.isView(sample), false);
}, null, ["passthrough"]);
