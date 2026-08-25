// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: get constructor on SpeciesConstructor
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

  1. Assert: Type(O) is Object.
  2. Let C be ? Get(O, "constructor").
  3. If C is undefined, return defaultConstructor.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));
  var calls = 0;
  var result;

  Object.defineProperty(sample, "constructor", {
    get: function() {
      calls++;
    }
  });

  result = sample.map(function() { return 0n; });

  assert.sameValue(calls, 1, "called custom ctor get accessor once");

  assert.sameValue(
    Object.getPrototypeOf(result),
    Object.getPrototypeOf(sample),
    "use defaultCtor on an undefined return - getPrototypeOf check"
  );
  assert.sameValue(
    result.constructor,
    TA,
    "use defaultCtor on an undefined return - .constructor check"
  );
});
