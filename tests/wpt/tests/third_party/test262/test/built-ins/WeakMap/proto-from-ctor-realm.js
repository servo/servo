// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: Default [[Prototype]] value derived from realm of the newTarget
info: |
    [...]
    2. Let map be ? OrdinaryCreateFromConstructor(NewTarget,
       "%WeakMapPrototype%", « [[WeakMapData]] »).
    [...]

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    [...]
features: [cross-realm, Reflect]
---*/

var other = $262.createRealm().global;
var C = new other.Function();
C.prototype = null;

var o = Reflect.construct(WeakMap, [], C);

assert.sameValue(Object.getPrototypeOf(o), other.WeakMap.prototype);
