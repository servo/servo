// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype-@@split
description: Default [[Prototype]] value derived from realm of the constructor
info: |
    10. Let splitter be ? Construct(C, « rx, newFlags »).

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    [...]
features: [cross-realm, Symbol.species, Symbol.split]
---*/

var other = $262.createRealm().global;
other.shared = null;
var C = new other.Function('shared = this; return /./;');
C.prototype = null;

var r = /./;
r.constructor = function() {};
r.constructor[Symbol.species] = C;

r[Symbol.split]();

assert.sameValue(Object.getPrototypeOf(other.shared), other.Object.prototype);
