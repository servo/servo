// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getfunctionrealm
description: >
  The realm of a bound function exotic object is the realm of its target function.
  GetFunctionRealm is called recursively.
info: |
  Object ( [ value ] )

  1. If NewTarget is neither undefined nor the active function, then
    a. Return ? OrdinaryCreateFromConstructor(NewTarget, "%Object.prototype%").

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  [...]
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return OrdinaryObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  [...]
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.

  GetFunctionRealm ( obj )

  [...]
  2. If obj has a [[Realm]] internal slot, then
    a. Return obj.[[Realm]].
  3. If obj is a bound function exotic object, then
    a. Let target be obj.[[BoundTargetFunction]].
    b. Return ? GetFunctionRealm(target).
features: [cross-realm, Reflect]
---*/

var realm1 = $262.createRealm().global;
var realm2 = $262.createRealm().global;
var realm3 = $262.createRealm().global;
var realm4 = $262.createRealm().global;

var newTarget = new realm1.Function();
newTarget.prototype = 1;

var boundNewTarget = realm2.Function.prototype.bind.call(newTarget);
var boundBoundNewTarget = realm3.Function.prototype.bind.call(boundNewTarget);
var object = Reflect.construct(realm4.Object, [], boundBoundNewTarget);

assert(object instanceof realm1.Object);
assert.sameValue(Object.getPrototypeOf(object), realm1.Object.prototype);
