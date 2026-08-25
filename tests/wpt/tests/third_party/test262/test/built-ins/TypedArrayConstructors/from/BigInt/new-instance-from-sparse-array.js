// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Throws a TypeError casting undefined value from sparse array to BigInt
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var source = [,42n];

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.throws(TypeError, function() {
    TA.from(source);
  });
}, null, ["passthrough"]);
