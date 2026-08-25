// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.of
description: >
  Calls and return abrupt from custom constructor
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
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var called = 0;
  var ctor = function() {
    called++;
    throw new Test262Error();
  };

  assert.throws(Test262Error, function() {
    TA.of.call(ctor, 42);
  });

  assert.sameValue(called, 1);
}, null, ["passthrough"]);
