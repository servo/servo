// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluatebody
es6id: 14.4.11
description: >
    Default [[Prototype]] value derived from realm of the generator function
info: |
    1. Let G be ? OrdinaryCreateFromConstructor(functionObject,
       "%GeneratorPrototype%", « [[GeneratorState]], [[GeneratorContext]] »).
    [...]

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    [...]
features: [generators, cross-realm]
---*/

var other = $262.createRealm().global;
var g = other.eval('(0, function*() {})');
var GeneratorPrototype = Object.getPrototypeOf(g.prototype);
g.prototype = null;
var instance;

instance = g();

assert.sameValue(Object.getPrototypeOf(instance), GeneratorPrototype);
