// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Find inherited properties if property is not a CanonicalNumericIndexString
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  4. Return ? OrdinaryHasProperty(O, P).
  ...
includes: [testTypedArray.js]
features: [Reflect, TypedArray]
---*/

TypedArray.prototype.foo = 42;
TypedArray.prototype[42] = true;

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  TA.prototype.bar = 42;

  assert.sameValue(Reflect.has(sample, "subarray"), true, "subarray");
  assert.sameValue(Reflect.has(sample, "foo"), true, "foo");
  assert.sameValue(Reflect.has(sample, "bar"), true, "bar");
  assert.sameValue(Reflect.has(sample, "baz"), false, "baz");

  assert.sameValue(Reflect.has(sample, "42"), false, "42");
});
