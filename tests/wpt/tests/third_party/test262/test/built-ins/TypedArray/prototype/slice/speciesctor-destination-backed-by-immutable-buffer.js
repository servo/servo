// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.slice
description: >
  Throws a TypeError exception when the receiver constructs an instance backed
  by an immutable buffer
info: |
  %TypedArray%.prototype.slice ( start, end )
  1. Let O be the this value.
  2. Let taRecord be ? ValidateTypedArray(O, seq-cst).
  3. Let srcArrayLength be TypedArrayLength(taRecord).
  4. Let relativeStart be ? ToIntegerOrInfinity(start).
  5. If relativeStart = -∞, let startIndex be 0.
  6. Else if relativeStart < 0, let startIndex be max(srcArrayLength + relativeStart, 0).
  7. Else, let startIndex be min(relativeStart, srcArrayLength).
  8. If end is undefined, let relativeEnd be srcArrayLength; else let relativeEnd be ? ToIntegerOrInfinity(end).
  9. If relativeEnd = -∞, let endIndex be 0.
  10. Else if relativeEnd < 0, let endIndex be max(srcArrayLength + relativeEnd, 0).
  11. Else, let endIndex be min(relativeEnd, srcArrayLength).
  12. Let countBytes be max(endIndex - startIndex, 0).
  13. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(countBytes) », ~write~).
  14. Assert: IsImmutableBuffer(A.[[ViewedArrayBuffer]]) is false.

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

  var start = {
    valueOf() {
      calls.push("start.valueOf");
      return 0;
    }
  };
  var end = {
    valueOf() {
      calls.push("end.valueOf");
      return 2;
    }
  };

  assert.throws(TypeError, function() {
    ta.slice(start, end);
  });
  var expectCalls = [
    "start.valueOf",
    "end.valueOf",
    "get ta.constructor",
    "get ta.constructor[Symbol.species]",
    "construct result",
    "return result"
  ];
  assert.compareArray(calls, expectCalls, "Must read arguments before constructing the result.");
});
