// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-length
description: Default [[Prototype]] value derived from realm of the newTarget
info: |
    [...]
    8. Return ? AllocateTypedArray(constructorName, NewTarget,
       "%TypedArrayPrototype%", elementLength).

    22.2.4.2.1 Runtime Semantics: AllocateTypedArray

    1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
    [...]

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    5. Return proto.
includes: [testTypedArray.js]
features: [cross-realm, Reflect, TypedArray]
---*/

var other = $262.createRealm().global;
var C = new other.Function();
C.prototype = null;

testWithTypedArrayConstructors(function(TA) {
  var ta = Reflect.construct(TA, [0], C);

  assert.sameValue(Object.getPrototypeOf(ta), other[TA.name].prototype);
}, null, ["passthrough"]);
