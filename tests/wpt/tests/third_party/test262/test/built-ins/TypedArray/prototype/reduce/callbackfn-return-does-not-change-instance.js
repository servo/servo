// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  The callbackfn return does not change the `this` instance
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([0, 1, 0]));

  sample.reduce(function() {
    return 42;
  }, 7);

  assert.sameValue(sample[0], 0, "[0] == 0");
  assert.sameValue(sample[1], 1, "[1] == 1");
  assert.sameValue(sample[2], 0, "[2] == 0");
});
