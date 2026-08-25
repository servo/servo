// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.move
description: The stack's [[DisposableState]] internal slot is set to disposed.
info: |
  DisposableStack.prototype.move ( )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. Let newDisposableStack be ? OrdinaryCreateFromConstructor(%DisposableStack%, "%DisposableStack.prototype%", « [[DisposableState]], [[DisposeCapability]] »).
  5. Set newDisposableStack.[[DisposableState]] to pending.
  6. Set newDisposableStack.[[DisposeCapability]] to disposableStack.[[DisposeCapability]].
  7. Set disposableStack.[[DisposeCapability]] to NewDisposeCapability().
  8. Set disposableStack.[[DisposableState]] to disposed.
  9. Return newDisposableStack.

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
var wasDisposed = stack.disposed;
stack.move();
var isDisposed = stack.disposed;
assert.sameValue(wasDisposed, false);
assert.sameValue(isDisposed, true);
