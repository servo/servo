// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Verify arguments on custom @@species construct call
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  10. Let A be ? TypedArraySpeciesCreate(O, « captured »).
  ...

  22.2.4.7 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  3. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
  4. Return ? TypedArrayCreate(constructor, argumentList).

  7.3.20 SpeciesConstructor ( O, defaultConstructor )

  ...
  5. Let S be ? Get(C, @@species).
  ...
  7. If IsConstructor(S) is true, return S.
  ...

  22.2.4.6 TypedArrayCreate ( constructor, argumentList )

  1. Let newTypedArray be ? Construct(constructor, argumentList).
  2. Perform ? ValidateTypedArray(newTypedArray).
  3. If argumentList is a List of a single Number, then
    ...
  4. Return newTypedArray.
includes: [testTypedArray.js]
features: [BigInt, Symbol.species, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 42n, 42n]));
  var result, ctorThis;

  sample.constructor = {};
  sample.constructor[Symbol.species] = function(count) {
    result = arguments;
    ctorThis = this;
    return new TA(count);
  };

  sample.filter(function(v) { return v === 42n; });

  assert.sameValue(result.length, 1, "called with 1 argument");
  assert.sameValue(result[0], 2, "[0] is the new captured length");

  assert(
    ctorThis instanceof sample.constructor[Symbol.species],
    "`this` value in the @@species fn is an instance of the function itself"
  );
});
