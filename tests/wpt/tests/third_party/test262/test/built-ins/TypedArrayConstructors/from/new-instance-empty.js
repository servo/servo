// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Return a new empty TypedArray
includes: [testTypedArray.js]
features: [TypedArray]
---*/


testWithTypedArrayConstructors(function(TA) {
  var result = TA.from([]);
  assert.sameValue(result.length, 0);
  assert.sameValue(result.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype);
}, null, ["passthrough"]);
