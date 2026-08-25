// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  Returns abrupt from callbackfn
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));

  assert.throws(Test262Error, function() {
    sample.map(function() {
      throw new Test262Error();
    });
  });
});
