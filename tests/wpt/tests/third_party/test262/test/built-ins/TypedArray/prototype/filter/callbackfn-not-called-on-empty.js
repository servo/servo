// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  callbackfn is not called on empty instances
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  9. Repeat, while k < len
    ...
    c. Let selected be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var called = 0;

  new TA().filter(function() {
    called++;
  });

  assert.sameValue(called, 0);
}, null, ["passthrough"]);
