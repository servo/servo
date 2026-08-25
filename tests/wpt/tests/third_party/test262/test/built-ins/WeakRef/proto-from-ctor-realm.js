// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: Default [[Prototype]] value derived from realm of the newTarget
info: |
  WeakRef( target )

  ...
  3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget,  '%WeakRefPrototype%', « [[Target]] »).
  4. Perfom ! KeepDuringJob(target).
  5. Set weakRef.[[Target]] to target.
  6. Return weakRef.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  3. Let proto be ? Get(constructor, 'prototype').
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [WeakRef, cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var wr;

newTarget.prototype = undefined;
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), other.WeakRef.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), other.WeakRef.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), other.WeakRef.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), other.WeakRef.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), other.WeakRef.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), other.WeakRef.prototype, 'newTarget.prototype is a Number');

