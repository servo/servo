// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: >
  Return abrupt completion getting newTarget's prototype
info: |
  22.2.4.3 TypedArray ( typedArray )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has a [[TypedArrayName]] internal slot.

  ...
  4. Let O be ? AllocateTypedArray(constructorName, NewTarget,
  %TypedArrayPrototype%).
  ...

  22.2.4.2.1 Runtime Semantics: AllocateTypedArray (constructorName, newTarget,
  defaultProto [ , length ])

  1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
  ...

  9.1.15 GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  ...
includes: [testTypedArray.js]
features: [BigInt, Reflect, TypedArray]
---*/

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get() {
    throw new Test262Error();
  }
});

var sample = new BigInt64Array();

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.throws(Test262Error, function() {
    Reflect.construct(TA, [sample], newTarget);
  });
}, null, ["passthrough"]);
