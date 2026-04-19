// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.move
description: Resources in the stack are not disposed when move is called.
info: |
  AsyncDisposableStack.prototype.move ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  4. Let newAsyncDisposableStack be ? OrdinaryCreateFromConstructor(%AsyncDisposableStack%, "%AsyncDisposableStack.prototype%", « [[AsyncDisposableState]], [[DisposeCapability]] »).
  5. Set newAsyncDisposableStack.[[AsyncDisposableState]] to pending.
  6. Set newAsyncDisposableStack.[[DisposeCapability]] to asyncDisposableStack.[[DisposeCapability]].
  7. Set asyncDisposableStack.[[DisposeCapability]] to NewDisposeCapability().
  8. Set asyncDisposableStack.[[AsyncDisposableState]] to disposed.
  9. Return newAsyncDisposableStack.

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();
var disposed = false;
stack.defer(() => { disposed = true; });
stack.move();
assert.sameValue(disposed, false);
