// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry-target
description: >
  The [[Prototype]] internal slot is computed from NewTarget.
info: |
  FinalizationRegistry ( cleanupCallback )

  ...
  3. Let finalizationRegistry be ? OrdinaryCreateFromConstructor(NewTarget,  "%FinalizationRegistryPrototype%", « [[Realm]], [[CleanupCallback]], [[Cells]], [[IsFinalizationRegistryCleanupJobActive]] »).
  ...
  9. Return finalizationRegistry.

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
features: [FinalizationRegistry, Reflect.construct]
---*/

var finalizationRegistry;

finalizationRegistry = Reflect.construct(FinalizationRegistry, [function() {}], Object);
assert.sameValue(Object.getPrototypeOf(finalizationRegistry), Object.prototype, 'NewTarget is built-in Object constructor');

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {
  get: function() {
    return Array.prototype;
  }
});
finalizationRegistry = Reflect.construct(FinalizationRegistry, [function() {}], newTarget);
assert.sameValue(Object.getPrototypeOf(finalizationRegistry), Array.prototype, 'NewTarget is BoundFunction with accessor');
