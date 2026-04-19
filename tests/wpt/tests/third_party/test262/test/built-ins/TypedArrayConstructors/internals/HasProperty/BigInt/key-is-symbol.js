// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Return boolean from Symbol properties
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    ...
  4. Return ? OrdinaryHasProperty(O, P).
includes: [testTypedArray.js]
features: [BigInt, Reflect, Symbol, TypedArray]
---*/

var s = Symbol("foo");

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  assert.sameValue(Reflect.has(sample, s), false);

  Object.defineProperty(sample, s, { value: 42 });

  assert.sameValue(Reflect.has(sample, s), true);
}, null, ["passthrough"]);
