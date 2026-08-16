// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.map
description: >
  Throws a TypeError exception when the receiver constructs an instance backed
  by an immutable buffer
info: |
  %TypedArray%.prototype.map ( callback [ , thisArg ] )
  1. Let O be the this value.
  2. Let taRecord be ? ValidateTypedArray(O, seq-cst).
  3. Let len be TypedArrayLength(taRecord).
  4. If IsCallable(callback) is false, throw a TypeError exception.
  5. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(len) », ~write~).
  6. Assert: IsImmutableBuffer(A.[[ViewedArrayBuffer]]) is false.

  TypedArraySpeciesCreate ( exemplar, argumentList [ , accessMode ] )
  1. If accessMode is not present, set accessMode to ~read~.
  2. Let defaultConstructor be the intrinsic object associated with the constructor name exemplar.[[TypedArrayName]] in Table 73.
  3. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
  4. Let result be ? TypedArrayCreateFromConstructor(constructor, argumentList, accessMode).

  TypedArrayCreateFromConstructor ( constructor, argumentList [ , accessMode ] )
  1. If accessMode is not present, set accessMode to ~read~.
  2. Let newTypedArray be ? Construct(constructor, argumentList).
  3. Let taRecord be ? ValidateTypedArray(newTypedArray, seq-cst, accessMode).

  ValidateTypedArray ( O, order [ , accessMode ] )
  1. If accessMode is not present, set accessMode to read.
  2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
  3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
  4. If accessMode is ~write~ and IsImmutableBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
features: [Symbol.species, TypedArray, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = [];

  var ta = new TA(makeCtorArg(["1", "2"]));
  var iab = (new TA(["3", "4"])).buffer.transferToImmutable();
  var constructor = {};
  Object.defineProperty(ta, "constructor", {
    get: function() {
      calls.push("get ta.constructor");
      return constructor;
    }
  });
  Object.defineProperty(constructor, Symbol.species, {
    get: function() {
      calls.push("get ta.constructor[Symbol.species]");
      return function speciesCtor() {
        calls.push("construct result");
        var result = new TA(iab);
        calls.push("return result");
        return result;
      };
    }
  });

  assert.throws(TypeError, function() {
    ta.map(function(value, index) {
      calls.push("map index " + index);
      return value + value;
    });
  });
  var expectCalls = [
    "get ta.constructor",
    "get ta.constructor[Symbol.species]",
    "construct result",
    "return result"
  ];
  assert.compareArray(calls, expectCalls, "Must construct the result before visiting elements.");
});
