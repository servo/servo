// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray
description: >
  Return abrupt completion getting newTarget's prototype
info: |
  22.2.4.1 TypedArray( )

  This description applies only if the TypedArray function is called with no
  arguments.

  ...
  3. Return ? AllocateTypedArray(constructorName, NewTarget,
  %TypedArrayPrototype%, 0).

  22.2.4.2.1 Runtime Semantics: AllocateTypedArray (constructorName, newTarget,
  defaultProto [ , length ])

  1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
  ...

  9.1.15 GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  ...
includes: [testTypedArray.js]
features: [Reflect, TypedArray]
---*/

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get() {
    throw new Test262Error();
  }
});

testWithTypedArrayConstructors(function(TA) {
  assert.throws(Test262Error, function() {
    Reflect.construct(TA, [], newTarget);
  });
}, null, ["passthrough"]);
