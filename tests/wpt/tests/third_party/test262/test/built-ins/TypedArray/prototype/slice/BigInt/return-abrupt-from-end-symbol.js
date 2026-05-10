// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: Return abrupt from ToInteger(end), end is symbol
info: |
  22.2.3.24 %TypedArray%.prototype.slice ( start, end )

  ...
  6. If end is undefined, let relativeEnd be len; else let relativeEnd be ?
  ToInteger(end).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

var s = Symbol("1");

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA();

  assert.throws(TypeError, function() {
    sample.slice(0, s);
  });
}, null, ["passthrough"]);
