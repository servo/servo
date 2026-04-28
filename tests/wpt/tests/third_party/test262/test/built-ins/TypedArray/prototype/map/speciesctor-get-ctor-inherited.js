// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: get inherited constructor on SpeciesConstructor
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
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));
  var calls = 0;
  var result;

  Object.defineProperty(TA.prototype, "constructor", {
    get: function() {
      calls++;
    }
  });

  result = sample.map(function() {
    return 0;
  });

  assert.sameValue(calls, 1, "called custom ctor get accessor once");

  assert.sameValue(
    Object.getPrototypeOf(result),
    Object.getPrototypeOf(sample),
    "use defaultCtor on an undefined return - getPrototypeOf check"
  );
  assert.sameValue(
    result.constructor,
    undefined,
    "used defaultCtor but still checks the inherited .constructor"
  );

  calls = 6;
  result.constructor;
  assert.sameValue(
    calls,
    7,
    "result.constructor triggers the inherited accessor property"
  );
}, null, ["passthrough"]);
