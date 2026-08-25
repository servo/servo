// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Use OrdinarySet if numeric key is not a CanonicalNumericIndex
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
  ...
  3. Return ? OrdinarySet(O, P, V, Receiver).
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, TypedArray]
---*/

var keys = [
  "1.0",
  "+1",
  "1000000000000000000000",
  "0.0000001"
];

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  keys.forEach(function(key) {
    var sample = new TA(makeCtorArg([42]));

    assert.sameValue(
      Reflect.set(sample, key, "ecma262"),
      true,
      'Reflect.set(sample, key, "ecma262") must return true'
    );
    assert.sameValue(sample[key], "ecma262", 'The value of sample[key] is "ecma262"');

    assert.sameValue(
      Reflect.set(sample, key, "es3000"),
      true,
      'Reflect.set(sample, key, "es3000") must return true'
    );
    assert.sameValue(sample[key], "es3000", 'The value of sample[key] is "es3000"');

    Object.defineProperty(sample, key, {
      writable: false,
      value: undefined
    });
    assert.sameValue(
      Reflect.set(sample, key, 42),
      false,
      'Reflect.set(sample, key, 42) must return false'
    );
    assert.sameValue(
      sample[key], undefined, 'The value of sample[key] is expected to equal `undefined`'
    );
  });
}, null, ["passthrough"]);
