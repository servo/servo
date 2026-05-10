// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns an ordinary property value if key is not a CanonicalNumericIndex
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

  assert.sameValue(
    Reflect.defineProperty(sample, "foo", {value:42}),
    true,
    "return true after defining property"
  );

  assert.sameValue(sample.foo, 42);
  verifyNotWritable(sample, "foo");
  verifyNotConfigurable(sample, "foo");
  verifyNotEnumerable(sample, "foo");

  var fnset = function() {};
  var fnget = function() {};
  assert.sameValue(
    Reflect.defineProperty(sample, "bar", {
      get: fnget,
      set: fnset,
      enumerable: false,
      configurable: true
    }),
    true,
    "return true after defining property"
  );

  var desc = Object.getOwnPropertyDescriptor(sample, "bar");
  assert.sameValue(desc.get, fnget, "accessor's get");
  assert.sameValue(desc.set, fnset, "accessor's set");
  verifyNotEnumerable(sample, "bar");
  verifyConfigurable(sample, "bar");
}, null, ["passthrough"]);
