// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getfunctionrealm
description: >
  The realm of a bound function exotic object is the realm of its target function.
info: |
  Date ( )

  [...]
  3. If NewTarget is undefined, then
    [...]
  4. Else,
    a. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%Date.prototype%", « [[DateValue]] »).
    [...]
    c. Return O.

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

var newTarget = new realm1.Function();
newTarget.prototype = "str";

var boundNewTarget = realm2.Function.prototype.bind.call(newTarget);
var date = Reflect.construct(realm3.Date, [], boundNewTarget);

assert(date instanceof realm1.Date);
assert.sameValue(Object.getPrototypeOf(date), realm1.Date.prototype);
