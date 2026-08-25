// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Return a new TypedArray using mapfn
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var mapfn = function(kValue) {
    return kValue * 2n;
  };

  var result = TA.from([42n, 43n, 42n], mapfn);
  assert.sameValue(result.length, 3);
  assert.sameValue(result[0], 84n);
  assert.sameValue(result[1], 86n);
  assert.sameValue(result[2], 84n);
  assert.sameValue(result.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype);
}, null, ["passthrough"]);
