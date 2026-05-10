// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Does not throw on an instance with a detached buffer if key is a Symbol
info: |
  9.4.5.4 [[Get]] (P, Receiver)

  ...
  2. If Type(P) is String, then
    ...
  3. Return ? OrdinaryGet(O, P, Receiver).
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA([42n, 43n]);
  $DETACHBUFFER(sample.buffer);

  var s = Symbol("1");

  assert.sameValue(sample[s], undefined);

  sample[s] = "test262";
  assert.sameValue(sample[s], "test262");
}, null, ["passthrough"]);
