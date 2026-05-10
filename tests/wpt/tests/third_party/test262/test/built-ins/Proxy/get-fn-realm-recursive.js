// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getfunctionrealm
description: >
  The realm of a Proxy exotic object is the realm of its target function.
  GetFunctionRealm is called recursively.
info: |
  Boolean ( value )

  [...]
  3. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%Boolean.prototype%", « [[BooleanData]] »).
  [...]
  5. Return O.

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
  [...]
  4. If obj is a Proxy exotic object, then
    [...]
    b. Let proxyTarget be obj.[[ProxyTarget]].
    c. Return ? GetFunctionRealm(proxyTarget).
features: [cross-realm, Reflect, Proxy]
---*/

var realm1 = $262.createRealm().global;
var realm2 = $262.createRealm().global;
var realm3 = $262.createRealm().global;
var realm4 = $262.createRealm().global;

var newTarget = new realm1.Function();
newTarget.prototype = null;

var newTargetProxy = new realm2.Proxy(newTarget, {});
var newTargetProxyProxy = new realm3.Proxy(newTargetProxy, {});
var boolean = Reflect.construct(realm4.Boolean, [], newTargetProxyProxy);

assert(boolean instanceof realm1.Boolean);
assert.sameValue(Object.getPrototypeOf(boolean), realm1.Boolean.prototype);
