// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Redefine a non-numerical key on a non-extensible instance
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  4. Return OrdinaryDefineOwnProperty(O, P, Desc).
  ...
includes: [testTypedArray.js, propertyHelper.js]
features: [Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43]));
  sample.foo = true;
  sample.bar = true;

  Object.preventExtensions(sample);

  assert.sameValue(
    Reflect.defineProperty(sample, "foo", {value:42}),
    true,
    "data descriptor"
  );

  assert.sameValue(sample.foo, 42);
  verifyEnumerable(sample, "foo");
  verifyWritable(sample, "foo");
  verifyConfigurable(sample, "foo");

  var fnget = function() {};
  var fnset = function() {};

  assert.sameValue(
    Reflect.defineProperty(sample, "bar", {
      get: fnget,
      set: fnset,
      enumerable: false,
      configurable: false
    }),
    true,
    "accessor descriptor"
  );

  var desc = Object.getOwnPropertyDescriptor(sample, "bar");
  assert.sameValue(desc.get, fnget, "accessor's get");
  assert.sameValue(desc.set, fnset, "accessor's set");
  verifyNotEnumerable(sample, "bar");
  verifyNotConfigurable(sample, "bar");
}, null, ["passthrough"]);
