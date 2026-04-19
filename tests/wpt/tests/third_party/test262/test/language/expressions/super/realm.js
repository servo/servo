// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: >
    Default [[Prototype]] value derived from realm of the newTarget value
info: |
    1. Let newTarget be GetNewTarget().
    [...]
    6. Let result be ? Construct(func, argList, newTarget).
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

class B extends function() {} {
  constructor() {
    super();
  }
}

var b = Reflect.construct(B, [], C);

assert.sameValue(Object.getPrototypeOf(b), other.Object.prototype);
