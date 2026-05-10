// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.move
description: Returns a new DisposableStack that contains the resources originally contained in this stack.
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

includes: [deepEqual.js]
features: [explicit-resource-management]
---*/

var stack1 = new DisposableStack();
var disposed = [];
stack1.defer(() => { disposed.push(1); });
stack1.defer(() => { disposed.push(2); });

var stack2 = stack1.move();

var wasDisposed = disposed.slice();
stack2.dispose();
var isDisposed = disposed.slice();

assert.deepEqual(wasDisposed, []);
assert.deepEqual(isDisposed, [2, 1]);
