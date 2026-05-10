// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: >
  The [[Prototype]] internal slot is computed from NewTarget.
info: |
  WeakRef ( target )

  ...
  3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget,  "%WeakRefPrototype%", « [[Target]] »).
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
features: [WeakRef, Reflect.construct]
---*/

var wr;

wr = Reflect.construct(WeakRef, [{}], Object);
assert.sameValue(Object.getPrototypeOf(wr), Object.prototype, 'NewTarget is built-in Object constructor');

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {
  get: function() {
    return Array.prototype;
  }
});
wr = Reflect.construct(WeakRef, [{}], newTarget);
assert.sameValue(Object.getPrototypeOf(wr), Array.prototype, 'NewTarget is BoundFunction with accessor');
