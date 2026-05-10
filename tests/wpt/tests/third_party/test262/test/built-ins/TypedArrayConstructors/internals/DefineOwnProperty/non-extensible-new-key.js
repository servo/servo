// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Can't define a new non-numerical key on a non-extensible instance
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
features: [Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43]));
  Object.preventExtensions(sample);

  assert.sameValue(
    Reflect.defineProperty(sample, "foo", {value:42}),
    false,
    "return false on a non-extensible object - data descriptor"
  );

  assert.sameValue(Object.getOwnPropertyDescriptor(sample, "foo"), undefined);

  assert.sameValue(
    Reflect.defineProperty(sample, "bar", {
      get: function() {},
      set: function() {},
      enumerable: false,
      configurable: true
    }),
    false,
    "return false on a non-extensible object - accessor descriptor"
  );

  assert.sameValue(Object.getOwnPropertyDescriptor(sample, "bar"), undefined);
}, null, ["passthrough"]);
