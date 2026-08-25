// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Return boolean from properties that are not a CanonicalNumericIndexString
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  4. Return ? OrdinaryHasProperty(O, P).
  ...
includes: [testTypedArray.js]
features: [BigInt, Reflect, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  assert.sameValue(Reflect.has(sample, "foo"), false);

  Object.defineProperty(sample, "foo", { value: 42 });

  assert.sameValue(Reflect.has(sample, "foo"), true);
}, null, ["passthrough"]);
