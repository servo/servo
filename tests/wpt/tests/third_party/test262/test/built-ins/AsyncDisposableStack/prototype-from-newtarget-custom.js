// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack
description: >
  The [[Prototype]] internal slot is computed from NewTarget.
info: |
  AsyncDisposableStack ( )

  ...
  2. Let asyncDisposableStack be ? OrdinaryCreateFromConstructor(NewTarget, "%AsyncDisposableStack.prototype%", « [[AsyncDisposableState]], [[DisposeCapability]] »).
  3. Set asyncDisposableStack.[[AsyncDisposableState]] to pending.
  4. Set asyncDisposableStack.[[DisposeCapability]] to NewDisposeCapability().
  5. Return asyncDisposableStack.

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

stack = Reflect.construct(AsyncDisposableStack, [], Object);
assert.sameValue(Object.getPrototypeOf(stack), Object.prototype, 'NewTarget is built-in Object constructor');

var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {
  get: function() {
    return Array.prototype;
  }
});
stack = Reflect.construct(AsyncDisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), Array.prototype, 'NewTarget is BoundFunction with accessor');
