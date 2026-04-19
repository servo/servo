// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-p1-p2-pn-body
description: Default [[Prototype]] value derived from realm of the newTarget
info: |
    [...]
    5. Return ? CreateDynamicFunction(C, NewTarget, "normal", args).

    19.2.1.1.1 Runtime Semantics: CreateDynamicFunction

    [...]
    2. If kind is "normal", then
       [...]
       c. Let fallbackProto be "%FunctionPrototype%".
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
features: [cross-realm, Reflect]
---*/

var other = $262.createRealm().global;
var C = new other.Function();
C.prototype = null;

var o = Reflect.construct(Function, [], C);

assert.sameValue(Object.getPrototypeOf(o), other.Function.prototype);
