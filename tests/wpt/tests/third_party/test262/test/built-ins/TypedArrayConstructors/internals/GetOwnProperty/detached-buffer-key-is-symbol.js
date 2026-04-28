// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-getownproperty-p
description: >
  Does not throw on an instance with a detached buffer if key is a Symbol
info: |
  9.4.5.1 [[GetOwnProperty]] ( P )

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
  4. Return OrdinaryGetOwnProperty(O, P).
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA([42, 43]);
  $DETACHBUFFER(sample.buffer);

  var s = Symbol("foo");
  Object.defineProperty(sample, s, { value: "baz" });
  assert.sameValue(
    Object.getOwnPropertyDescriptor(sample, s).value,
    "baz",
    "return value from a Symbol key"
  );
}, null, ["passthrough"]);
