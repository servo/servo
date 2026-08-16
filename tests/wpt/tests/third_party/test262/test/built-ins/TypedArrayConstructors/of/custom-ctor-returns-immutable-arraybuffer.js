// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.of
description: >
  Throws a TypeError if species constructor returns an immutable ArrayBuffer.
info: |
  %TypedArray%.of ( ...items )
  1. Let len be the number of elements in items.
  2. Let C be the this value.
  3. If IsConstructor(C) is false, throw a TypeError exception.
  4. Let newObj be ? TypedArrayCreateFromConstructor(C, « 𝔽(len) », write).
  5. Let k be 0.
  6. Repeat, while k < len,
     a. Let kValue be items[k].
     b. Let Pk be ! ToString(𝔽(k)).
     c. Perform ? Set(newObj, Pk, kValue, true).
     d. Set k to k + 1.
features: [Symbol.iterator, TypedArray, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = [];

  var custom = new TA(makeCtorArg(2));
  var ctor = function(len) {
    calls.push("construct(" + len + ")");
    return custom;
  };

  var a = {
    valueOf() {
      calls.push("a.valueOf");
      return "1";
    }
  };
  var b = {
    valueOf() {
      calls.push("b.valueOf");
      return "2";
    }
  };
  assert.throws(TypeError, function() {
    TA.of.call(ctor, a, b);
  }, "iterable source");
  assert.compareArray(calls, ["construct(2)"]);
}, null, ["immutable"]);
