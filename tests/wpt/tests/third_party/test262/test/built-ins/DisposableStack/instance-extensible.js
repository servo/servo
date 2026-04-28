// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack
description: Instances of DisposableStack are extensible
info: |
  DisposableStack( )

  ...
  2. Let disposableStack be ? OrdinaryCreateFromConstructor(NewTarget, "%DisposableStack.prototype%", « [[DisposableState]], [[DisposeCapability]] »).
  3. Set disposableStack.[[DisposableState]] to pending.
  4. Set disposableStack.[[DisposeCapability]] to NewDisposeCapability().
  5. Return disposableStack.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  ObjectCreate ( proto [ , internalSlotsList ] )

  4. Set obj.[[Prototype]] to proto.
  5. Set obj.[[Extensible]] to true.
  6. Return obj.
features: [explicit-resource-management, Reflect]
---*/

var stack = new DisposableStack();
assert.sameValue(Object.isExtensible(stack), true);
