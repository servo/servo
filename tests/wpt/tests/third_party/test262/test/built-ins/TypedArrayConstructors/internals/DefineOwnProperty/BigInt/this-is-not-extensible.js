// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns false for non-numeric index property value if `this` is not extensible
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  4. Return OrdinaryDefineOwnProperty(O, P, Desc).
  ...
includes: [testTypedArray.js]
features: [BigInt, Reflect, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  Object.preventExtensions(sample);

  assert.sameValue(Reflect.defineProperty(sample, "foo", {value:42}), false);
  assert.sameValue(Reflect.getOwnPropertyDescriptor(sample, "foo"), undefined);

  var s = Symbol("1");
  assert.sameValue(Reflect.defineProperty(sample, s, {value:42}), false);
  assert.sameValue(Reflect.getOwnPropertyDescriptor(sample, s), undefined);
}, null, ["passthrough"]);
