// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.of
description: >
  Throws a TypeError if a custom `this` returns a smaller instance
info: |
  %TypedArray%.of ( ...items )

  1. Let len be the actual number of arguments passed to this function.
  ...
  5. Let newObj be ? TypedArrayCreate(C, « len »).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ctor = function() {
    return new TA(makeCtorArg(1));
  };

  assert.throws(TypeError, function() {
    TypedArray.of.call(ctor, 1, 2);
  });
});
