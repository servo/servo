// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Does not throw on an instance with a detached buffer if key is not a number
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  3. Return ? OrdinarySet(O, P, V, Receiver).
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, Reflect, TypedArray]
---*/
testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA(2);
  $DETACHBUFFER(sample.buffer);

  assert.sameValue(
    Reflect.set(sample, 'foo', 'test262'),
    true,
    'Reflect.set(sample, "foo", "test262") must return true'
  );

  assert.sameValue(sample.foo, 'test262', 'The value of sample.foo is "test262"');
}, null, ["passthrough"]);
