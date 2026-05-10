// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
  If trap is undefined, propagate [[Construct]] to target,
  passing correct newTarget parameter
info: |
  [[Construct]] ( argumentsList, newTarget )

  [...]
  7. If trap is undefined, then
    b. Return ? Construct(target, argumentsList, newTarget).

  Construct ( F [ , argumentsList [ , newTarget ] ] )

  [...]
  5. Return ? F.[[Construct]](argumentsList, newTarget).

  [[Construct]] ( argumentsList, newTarget )

  [...]
  5. If kind is "base", then
    a. Let thisArgument be ? OrdinaryCreateFromConstructor(newTarget, "%ObjectPrototype%").

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  [...]
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

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
features: [cross-realm, Proxy, Reflect, Reflect.construct]
---*/

var other = $262.createRealm().global;
var C = new other.Function();
C.prototype = null;

var P = new Proxy(function() {}, {});
var p = Reflect.construct(P, [], C);

assert.sameValue(Object.getPrototypeOf(p), other.Object.prototype);
