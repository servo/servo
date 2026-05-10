// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-getownproperty-p
description: >
  Returns a descriptor object from an index property
info: |
  9.4.5.1 [[GetOwnProperty]] ( P )

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
      iii. Return a PropertyDescriptor{[[Value]]: value, [[Writable]]: true,
      [[Enumerable]]: true, [[Configurable]]: true}.
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/
testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  let descriptor0 = Object.getOwnPropertyDescriptor(sample, "0");
  let descriptor1 = Object.getOwnPropertyDescriptor(sample, "1");

  assert.sameValue(descriptor0.value, 42n);
  assert.sameValue(descriptor0.configurable, true);
  assert.sameValue(descriptor0.enumerable, true);
  assert.sameValue(descriptor0.writable, true);

  assert.sameValue(descriptor1.value, 43n);
  assert.sameValue(descriptor1.configurable, true);
  assert.sameValue(descriptor1.enumerable, true);
  assert.sameValue(descriptor1.writable, true);
});
