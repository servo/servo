// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Return a new TypedArray from an ordinary object
includes: [testTypedArray.js]
features: [BigInt, Array.prototype.values, TypedArray]
---*/

var source = {
  "0": 42n,
  "1": 44n,
  length: 2
};

testWithBigIntTypedArrayConstructors(function(TA) {
  var result = TA.from(source);

  assert.sameValue(result.length, 2);
  assert.sameValue(result[0], 42n);
  assert.sameValue(result[1], 44n);
  assert.sameValue(result.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype);
}, null, ["passthrough"]);
