// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.defer
description: Adds a disposable resource to the stack
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
var disposed = [];
function dispose1() { disposed.push(dispose1); }
function dispose2() { disposed.push(dispose2); }
stack.defer(dispose1);
stack.defer(dispose2);
stack.dispose();
assert.sameValue(2, disposed.length);
assert.sameValue(disposed[0], dispose2, 'Expected dispose2 to be the first onDispose invoked');
assert.sameValue(disposed[1], dispose1, 'Expected dispose1 to be the second onDispose invoked');
