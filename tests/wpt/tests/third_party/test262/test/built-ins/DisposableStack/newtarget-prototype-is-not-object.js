// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack
description: >
  [[Prototype]] defaults to %DisposableStack.prototype% if NewTarget.prototype is not an object.
info: |
  DisposableStack( target )

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
features: [explicit-resource-management, Reflect.construct, Symbol]
---*/

var stack;
function newTarget() {}

newTarget.prototype = undefined;
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), DisposableStack.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), DisposableStack.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), DisposableStack.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), DisposableStack.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), DisposableStack.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
stack = Reflect.construct(DisposableStack, [], newTarget);
assert.sameValue(Object.getPrototypeOf(stack), DisposableStack.prototype, 'newTarget.prototype is a Number');
