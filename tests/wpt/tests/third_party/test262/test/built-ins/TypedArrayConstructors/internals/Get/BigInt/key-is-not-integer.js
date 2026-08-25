// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Return undefined if key is numeric index is not an integer.
info: |
  9.4.5.4 [[Get]] (P, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Return ? IntegerIndexedElementGet(O, numericIndex).
  ...

  9.4.5.8 IntegerIndexedElementGet ( O, index )

  ...
  5. If IsInteger(index) is false, return undefined.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var proto = TypedArray.prototype;
Object.defineProperty(proto, "1.1", {
  get: function() {
    throw new Test262Error("OrdinaryGet was called! Ref: 9.1.8.1 3.c");
  }
});

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  assert.sameValue(sample["1.1"], undefined);
}, null, ["passthrough"]);
