// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  Use defaultConstructor if @@species is either undefined or null
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

  ...
  6. Let A be ? TypedArraySpeciesCreate(O, « len »).
  ...

  22.2.4.7 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  3. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
  ...

  7.3.20 SpeciesConstructor ( O, defaultConstructor )

  ...
  5. Let S be ? Get(C, @@species).
  6. If S is either undefined or null, return defaultConstructor.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol.species, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));
  var result;

  sample.constructor = {};

  result = sample.map(function() { return 0n; });

  assert.sameValue(
    Object.getPrototypeOf(result),
    Object.getPrototypeOf(sample),
    "undefined @@species - prototype check "
  );
  assert.sameValue(result.constructor, TA, "undefined @@species - ctor check");

  sample.constructor[Symbol.species] = null;
  result = sample.map(function() { return 0n; });

  assert.sameValue(
    Object.getPrototypeOf(result),
    Object.getPrototypeOf(sample),
    "null @@species - prototype check "
  );
  assert.sameValue(result.constructor, TA, "null @@species - ctor check");
});
