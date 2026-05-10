// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Use OrdinaryGet if key is not a CanonicalNumericIndex
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

TypedArray.prototype.baz = "test262";

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  assert.sameValue(
    sample.foo, undefined,
    "return undefined for inexistent properties"
  );

  sample.foo = "bar";
  assert.sameValue(sample.foo, "bar", "return value");

  Object.defineProperty(sample, "bar", {
    get: function() { return "baz"; }
  });
  assert.sameValue(sample.bar, "baz", "return value from get accessor");

  assert.sameValue(sample.baz, "test262", "return value from inherited key");
}, null, ["passthrough"]);
