// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Use OrdinaryGet if numeric key is not a CanonicalNumericIndex
info: |
  9.4.5.4 [[Get]] (P, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  3. Return ? OrdinaryGet(O, P, Receiver).
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

var keys = [
  "1.0",
  "+1",
  "1000000000000000000000",
  "0.0000001"
];

testWithBigIntTypedArrayConstructors(function(TA) {
  keys.forEach(function(key) {
    var sample = new TA();

    assert.sameValue(
      sample[key], undefined,
      "return undefined for inexistent properties [" + key + "]"
    );

    TypedArray.prototype[key] = "test262";

    assert.sameValue(
      sample[key],
      "test262",
      "return value from inherited key [" + key + "]"
    );

    sample[key] = "bar";
    assert.sameValue(
      sample[key], "bar",
      "return value from own key [" + key + "]"
    );

    Object.defineProperty(sample, key, {
      get: function() { return "baz"; }
    });

    assert.sameValue(
      sample[key], "baz",
      "return value from get accessor [" + key + "]"
    );

    delete TypedArray.prototype[key];
  });
}, null, ["passthrough"]);
