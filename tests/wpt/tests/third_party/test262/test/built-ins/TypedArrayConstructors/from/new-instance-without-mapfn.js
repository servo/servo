// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Return a new TypedArray
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var source = [42, 43, 42];

testWithTypedArrayConstructors(function(TA) {
  var result = TA.from(source);
  assert.sameValue(result.length, 3);
  assert.sameValue(result[0], 42);
  assert.sameValue(result[1], 43);
  assert.sameValue(result[2], 42);
  assert.sameValue(result.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype);
}, null, ["passthrough"]);
