// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Does not throw on an instance with a detached buffer if key is not a number
info: |
  9.4.5.4 [[Get]] (P, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  3. Return ? OrdinaryGet(O, P, Receiver
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA([42n, 43n]);
  sample.foo = "test262";

  $DETACHBUFFER(sample.buffer);

  assert.sameValue(sample.undef, undefined);
  assert.sameValue(sample.foo, "test262");
}, null, ["passthrough"]);
