// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.defer
description: Adds an onDispose callback to the stack
info: |
  DisposableStack.prototype.defer ( onDispose )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. If IsCallable(onDispose) is false, throw a TypeError exception.
  5. Perform ? AddDisposableResource(disposableStack.[[DisposeCapability]], undefined, sync-dispose, onDispose).
  ...

  AddDisposableResource ( disposeCapability, V, hint [, method ] )

  1. If method is not present then,
    ...
  2. Else,
    a. Assert: V is undefined.
    b. Let resource be ? CreateDisposableResource(undefined, hint, method).
  3. Append resource to disposeCapability.[[DisposableResourceStack]].
  4. Return unused.

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
var disposed = false;
stack.defer(() => { disposed = true });
stack.dispose();
assert.sameValue(disposed, true, 'Expected callback to have been called');
