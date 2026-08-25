// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  Throws if O.constructor returns a non-Object and non-undefined value
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).

  22.2.4.7 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  3. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
  ...

  7.3.20 SpeciesConstructor ( O, defaultConstructor )

  1. Assert: Type(O) is Object.
  2. Let C be ? Get(O, "constructor").
  3. If C is undefined, return defaultConstructor.
  4. If Type(C) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  sample.constructor = 42;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "42");

  sample.constructor = "1";
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "string");

  sample.constructor = null;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "null");

  sample.constructor = NaN;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "NaN");

  sample.constructor = false;
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "false");

  sample.constructor = Symbol("1");
  assert.throws(TypeError, function() {
    sample.subarray(0);
  }, "symbol");
});
