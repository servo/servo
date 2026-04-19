// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: >
  Return abrupt from getting object property
info: |
  22.2.4.4 TypedArray ( object )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]]
  internal slot.

  ...
  8. Repeat, while k < len
    ...
    b. Let kValue be ? Get(arrayLike, Pk).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var obj = {
  length: 4
};

Object.defineProperty(obj, "2", {
  get() {
    throw new Test262Error();
  }
});

testWithTypedArrayConstructors(function(TA) {
  assert.throws(Test262Error, function() {
    new TA(obj);
  });
}, null, ["passthrough"]);
