// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: >
  Perform regular set if target's uses a different element type
info: |
  22.2.3.24 %TypedArray%.prototype.slice ( start, end )

  ...
  9. Let A be ? TypedArraySpeciesCreate(O, « count »).
  10. Let srcName be the String value of O's [[TypedArrayName]] internal slot.
  11. Let srcType be the String value of the Element Type value in Table 50 for
  srcName.
  12. Let targetName be the String value of A's [[TypedArrayName]] internal
  slot.
  13. Let targetType be the String value of the Element Type value in Table 50
  for targetName.
  14. If SameValue(srcType, targetType) is false, then
    a. Let n be 0.
    b. Repeat, while k < final
      i. Let Pk be ! ToString(k).
      ii. Let kValue be ? Get(O, Pk).
      iii. Perform ? Set(A, ! ToString(n), kValue, true).
      iv. Increase k by 1.
      v. Increase n by 1.
  ...
  16. Return A
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, Symbol.species, TypedArray]
---*/

var arr = [42n, 43n, 44n];

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(arr));
  var other = TA === BigInt64Array ? BigUint64Array : BigInt64Array;

  sample.constructor = {};
  sample.constructor[Symbol.species] = other;

  var result = sample.slice();

  assert(compareArray(result, arr), "values are set");
  assert.notSameValue(result.buffer, sample.buffer, "creates a new buffer");
  assert.sameValue(result.constructor, other, "used the custom ctor");
});
