// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: >
  Use default ArrayBuffer constructor on null buffer.constructor.@@species
info: |
  22.2.4.3 TypedArray ( typedArray )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has a [[TypedArrayName]] internal slot.

  ...
  18. Else,
    a. Let bufferConstructor be ? SpeciesConstructor(srcData, %ArrayBuffer%).
  ...

  7.3.20 SpeciesConstructor ( O, defaultConstructor )

  ...
  5. Let S be ? Get(C, @@species).
  6. If S is either undefined or null, return defaultConstructor.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol.species, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var OtherCtor = TA === BigInt64Array ? BigUint64Array : BigInt64Array;
  var sample = new OtherCtor();
  var ctor = {};

  sample.buffer.constructor = ctor;

  ctor[Symbol.species] = null;
  var typedArray = new TA(sample);

  assert.sameValue(
    Object.getPrototypeOf(typedArray.buffer),
    ArrayBuffer.prototype,
    "buffer ctor is not called when species is null"
  );
}, null, ["passthrough"]);
