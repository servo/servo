// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generatorfunction
description: Default [[Prototype]] value derived from realm of the newTarget
info: |
    [...]
    3. Return ? CreateDynamicFunction(C, NewTarget, "generator", args).

    19.2.1.1.1 Runtime Semantics: CreateDynamicFunction

    [...]
    3. Else,
       [...]
       c. Let fallbackProto be "%Generator%".
    [...]
    22. Let proto be ? GetPrototypeFromConstructor(newTarget, fallbackProto).
    [...]

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    [...]
features: [generators, cross-realm, Reflect]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;
var other = $262.createRealm().global;
var OtherGeneratorFunction = Object.getPrototypeOf(
  other.eval('(0, function* () {})')
).constructor;
var C = new other.Function();
C.prototype = null;

var o = Reflect.construct(GeneratorFunction, [], C);

assert.sameValue(Object.getPrototypeOf(o), OtherGeneratorFunction.prototype);
