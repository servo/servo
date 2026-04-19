// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack
description: Instances of AsyncDisposableStack are extensible
info: |
  AsyncDisposableStack( )

  ...
  2. Let asyncDisposableStack be ? OrdinaryCreateFromConstructor(NewTarget, "%AsyncDisposableStack.prototype%", « [[AsyncDisposableState]], [[DisposeCapability]] »).
  3. Set asyncDisposableStack.[[AsyncDisposableState]] to pending.
  4. Set asyncDisposableStack.[[DisposeCapability]] to NewDisposeCapability().
  5. Return asyncDisposableStack.

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

var stack = new AsyncDisposableStack();
assert.sameValue(Object.isExtensible(stack), true);
