// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: >
  Throws a TypeError if new typedArray's length < count
info: |
  23.2.3.27 %TypedArray%.prototype.slice ( start, end )

  ...
  9. Let A be ? TypedArraySpeciesCreate(O, « count »).
  ...

  23.2.4.1 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  3. Let result be ? TypedArrayCreate(constructor, argumentList).

  23.2.4.2 TypedArrayCreate ( constructor, argumentList )

  ...
  3. If argumentList is a List of a single Number, then
    a. If the value of newTypedArray's [[ArrayLength]] internal slot <
    argumentList[0], throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol.species, TypedArray, resizable-arraybuffer]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  const rab1 = new ArrayBuffer(8, {maxByteLength: 100});
  const ta = new TA(rab1);
  const rab2 = new ArrayBuffer(10, {maxByteLength: 20});
  const lengthTracking = new TA(rab2);
  rab2.resize(0);
  ta.constructor = { [Symbol.species]: function() { return lengthTracking; } };
  assert.throws(TypeError, function() {
    ta.slice();
  });
}, null, ["passthrough"]);
