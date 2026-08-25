// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Return a new TypedArray using -0 and +0
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var result = TA.from([-0, +0]);
  assert.sameValue(result.length, 2);
  assert.sameValue(result[0], -0, "-0 => -0");
  assert.sameValue(result[1], 0, "+0 => 0");
  assert.sameValue(result.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype);
},
floatArrayConstructors);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var result = TA.from([-0, +0]);
  assert.sameValue(result.length, 2);
  assert.sameValue(result[0], 0, "-0 => 0");
  assert.sameValue(result[1], 0, "+0 => 0");
  assert.sameValue(result.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype);
},
intArrayConstructors);
