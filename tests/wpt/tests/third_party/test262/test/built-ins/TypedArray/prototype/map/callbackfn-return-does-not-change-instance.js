// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  The callbackfn return does not change the `this` instance
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample1 = new TA(makeCtorArg(3));

  sample1[1] = 1;

  sample1.map(function() {
    return 42;
  });

  assert.sameValue(sample1[0], 0, "[0] == 0");
  assert.sameValue(sample1[1], 1, "[1] == 1");
  assert.sameValue(sample1[2], 0, "[2] == 0");
});
