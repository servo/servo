// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  Throws if returned @@species is not a constructor, null or undefined.
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).

  22.2.4.7 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  3. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
  ...

  7.3.20 SpeciesConstructor ( O, defaultConstructor )

  ...
  5. Let S be ? Get(C, @@species).
  6. If S is either undefined or null, return defaultConstructor.
  7. If IsConstructor(S) is true, return S.
  8. Throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol.species, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  sample.constructor = {};

  sample.constructor[Symbol.species] = 0;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "0");

  sample.constructor[Symbol.species] = "string";
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "string");

  sample.constructor[Symbol.species] = {};
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "{}");

  sample.constructor[Symbol.species] = NaN;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "NaN");

  sample.constructor[Symbol.species] = false;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "false");

  sample.constructor[Symbol.species] = true;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "true");
});
