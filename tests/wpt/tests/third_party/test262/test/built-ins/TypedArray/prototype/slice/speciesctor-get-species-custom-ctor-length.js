// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: >
  Does not throw a TypeError if new typedArray's length >= count
info: |
  22.2.3.24 %TypedArray%.prototype.slice ( start, end )

  ...
  9. Let A be ? TypedArraySpeciesCreate(O, « count »).
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
features: [Symbol.species, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));
  var customCount, result;

  sample.constructor = {};
  sample.constructor[Symbol.species] = function() {
    return new TA(customCount);
  };

  customCount = 2;
  result = sample.slice();
  assert.sameValue(result.length, customCount, "length == count");

  customCount = 5;
  result = sample.slice();
  assert.sameValue(result.length, customCount, "length > count");
});
