// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: >
  Return abrupt from getting object @@iterator
info: |
  22.2.4.4 TypedArray ( object )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]]
  internal slot.

  ...
  4. Let arrayLike be ? IterableToArrayLike(object).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol.iterator, TypedArray]
---*/

var obj = function () {};

Object.defineProperty(obj, Symbol.iterator, {
  get() {
    throw new Test262Error();
  }
});

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.throws(Test262Error, function() {
    new TA(obj);
  });
}, null, ["passthrough"]);
