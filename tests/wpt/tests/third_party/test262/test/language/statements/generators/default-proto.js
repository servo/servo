// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluatebody
es6id: 14.4.11
description: Intrinsic default prototype of GeneratorFunctions
info: |
    1. Let G be ? OrdinaryCreateFromConstructor(functionObject,
       "%GeneratorPrototype%", « [[GeneratorState]], [[GeneratorContext]] »).
    [...]

    9.1.13 OrdinaryCreateFromConstructor

    [...]
    2. Let proto be ? GetPrototypeFromConstructor(constructor,
       intrinsicDefaultProto).
    3. Return ObjectCreate(proto, internalSlotsList).

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    [...]
features: [generators]
---*/

function* g() {}
var GeneratorPrototype = Object.getPrototypeOf(g).prototype;
g.prototype = null;

assert.sameValue(Object.getPrototypeOf(g()), GeneratorPrototype);
