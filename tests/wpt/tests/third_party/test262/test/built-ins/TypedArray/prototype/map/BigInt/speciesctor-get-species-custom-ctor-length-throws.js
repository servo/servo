// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  Throws a TypeError if new typedArray's length < len
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

  ...
  6. Let A be ? TypedArraySpeciesCreate(O, « len »).
  ...

  22.2.4.7 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  4. Return ? TypedArrayCreate(constructor, argumentList).

  22.2.4.6 TypedArrayCreate ( constructor, argumentList )

  ...
  3. If argumentList is a List of a single Number, then
    a. If the value of newTypedArray's [[ArrayLength]] internal slot <
    argumentList[0], throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol.species, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  sample.constructor = {};
  sample.constructor[Symbol.species] = function() {
    return new TA();
  };

  assert.throws(TypeError, function() {
    sample.map(function() { return 0; });
  });
});
