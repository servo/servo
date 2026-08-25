// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  Verify arguments on custom @@species construct call
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  15. If O.[[ArrayLength]] is auto and end is undefined, then
      a. Let argumentsList be « buffer, 𝔽(beginByteOffset) ».
  16. Else,
      ...
      f. Let argumentsList be « buffer, 𝔽(beginByteOffset), 𝔽(newLength) ».
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).

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
includes: [compareArray.js, testTypedArray.js]
features: [Symbol.species, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42]));
  var expectedOffset = TA.BYTES_PER_ELEMENT;
  var result, ctorThis;

  sample.constructor = {};
  sample.constructor[Symbol.species] = function(buffer, offset, length) {
    result = arguments;
    ctorThis = this;
    return new TA(buffer, offset, length);
  };

  sample.subarray(1);

  var expectArgs = sample.buffer.resizable
    ? [sample.buffer, expectedOffset]
    : [sample.buffer, expectedOffset, 2];
  assert.compareArray(result, expectArgs, "Constructor called with arguments");

  assert(
    ctorThis instanceof sample.constructor[Symbol.species],
    "`this` value in the @@species fn is an instance of the function itself"
  );
});
