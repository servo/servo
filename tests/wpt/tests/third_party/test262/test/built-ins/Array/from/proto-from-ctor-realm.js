// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
es6id: 22.1.2.1
description: Default [[Prototype]] value derived from realm of the constructor
info: |
    [...]
    5. If usingIterator is not undefined, then
       a. If IsConstructor(C) is true, then
          i. Let A be ? Construct(C).
    [...]

    9.1.14 GetPrototypeFromConstructor

    [...]
    3. Let proto be ? Get(constructor, "prototype").
    4. If Type(proto) is not Object, then
       a. Let realm be ? GetFunctionRealm(constructor).
       b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
    [...]
features: [cross-realm]
---*/

var other = $262.createRealm().global;
var C = new other.Function();
C.prototype = null;

var a = Array.from.call(C, []);

assert.sameValue(
  Object.getPrototypeOf(a),
  other.Object.prototype,
  'Object.getPrototypeOf(Array.from.call(C, [])) returns other.Object.prototype'
);
