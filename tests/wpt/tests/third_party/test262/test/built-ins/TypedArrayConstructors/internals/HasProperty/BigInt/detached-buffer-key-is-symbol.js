// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Does not throw on an instance with a detached buffer if key is a Symbol
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  4. Return ? OrdinaryHasProperty(O, P).
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, Reflect, Symbol, TypedArray]
---*/

var s1 = Symbol("foo");
var s2 = Symbol("bar");

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA([42n, 43n]);
  Object.defineProperty(sample, s1, { value: "baz" });

  $DETACHBUFFER(sample.buffer);

  assert.sameValue(Reflect.has(sample, s1), true);
  assert.sameValue(Reflect.has(sample, s2), false);
}, null, ["passthrough"]);
