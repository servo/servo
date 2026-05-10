// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Return boolean from numeric keys that are not a CanonicalNumericIndexString
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

var keys = [
  "1.0",
  "+1",
  "1000000000000000000000",
  "0.0000001"
];

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  keys.forEach(function(key) {
    var sample = new TA(makeCtorArg(1));

    assert.sameValue(
      Reflect.has(sample, key), false,
      "returns false without key [" + key + "]"
    );

    TypedArray.prototype[key] = 42;

    assert.sameValue(
      Reflect.has(sample, key), true,
      "returns true with inherited key [" + key + "]"
    );

    delete TypedArray.prototype[key];

    Object.defineProperty(sample, key, {value: 42n});

    assert.sameValue(
      Reflect.has(sample, key), true,
      "returns true with own key [" + key + "]"
    );
  });
}, null, ["passthrough"]);
