// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  The callbackfn return does not change the instance
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample1 = new TA(makeCtorArg(3));

  sample1[1] = 1n;

  sample1.filter(function() {
    return 42;
  });

  assert.sameValue(sample1[0], 0n, "[0] == 0");
  assert.sameValue(sample1[1], 1n, "[1] == 1");
  assert.sameValue(sample1[2], 0n, "[2] == 0");
});
