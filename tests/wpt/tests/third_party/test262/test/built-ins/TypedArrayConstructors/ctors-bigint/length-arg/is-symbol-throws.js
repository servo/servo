// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-length
description: >
  If length is a Symbol, throw a TypeError exception.
info: |
  22.2.4.2 TypedArray ( length )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is not Object.

  ...
  4. Let numberLength be ? ToNumber(length).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

var s = Symbol('1');

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.throws(TypeError, function() {
    new TA(s);
  });
}, null, ["passthrough"]);
