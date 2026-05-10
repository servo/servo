// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.of
description: >
  Custom constructor needs to instantiate a TypedArray
info: |
  22.2.2.2 %TypedArray%.of ( ...items )

  ...
  5. Let newObj be ? TypedArrayCreate(C, «len»).
  ...

  22.2.4.6 TypedArrayCreate ( constructor, argumentList )

  1. Let newTypedArray be ? Construct(constructor, argumentList).
  2. Perform ? ValidateTypedArray(newTypedArray).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var ctor = function() {};

  assert.throws(TypeError, function() {
    TA.of.call(ctor, 42n);
  });
}, null, ["passthrough"]);
