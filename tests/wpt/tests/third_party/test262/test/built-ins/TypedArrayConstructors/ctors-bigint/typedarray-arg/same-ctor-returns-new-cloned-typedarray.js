// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: >
  Same typedArray ctor argument returns a new cloned typedArray
info: |
  22.2.4.3 TypedArray ( typedArray )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has a [[TypedArrayName]] internal slot.

  ...
  17. If SameValue(elementType, srcType) is true, then
    a. Let data be ? CloneArrayBuffer(srcData, srcByteOffset).
  ...
  23. Return O.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(7));
  var typedArray = new TA(sample);

  assert.sameValue(typedArray.length, 7);
  assert.notSameValue(typedArray, sample);
  assert.notSameValue(typedArray.buffer, sample.buffer);
  assert.sameValue(typedArray.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(typedArray), TA.prototype);
});
