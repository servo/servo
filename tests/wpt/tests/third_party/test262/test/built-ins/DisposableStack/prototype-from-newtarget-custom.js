// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack
description: >
  The [[Prototype]] internal slot is computed from NewTarget.
info: |
  DisposableStack ( )

  ...
  2. Let disposableStack be ? OrdinaryCreateFromConstructor(NewTarget, "%DisposableStack.prototype%", « [[DisposableState]], [[DisposeCapability]] »).
  3. Set disposableStack.[[DisposableState]] to pending.
  4. Set disposableStack.[[DisposeCapability]] to NewDisposeCapability().
  5. Return disposableStack.

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
features: [explicit-resource-management, Reflect.construct]
---*/

var stack;

stack = Reflect.construct(DisposableStack, [], Object);
assert.sameValue(Object.getPrototypeOf(stack), Object.prototype, 'NewTarget is built-in Object constructor');

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {
  get: function() {
    return Array.prototype;
  }
});
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), Array.prototype, 'NewTarget is BoundFunction with accessor');
