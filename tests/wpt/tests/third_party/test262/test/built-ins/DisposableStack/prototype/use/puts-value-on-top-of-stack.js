// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Puts value on the top of the dispose stack
info: |
  DisposableStack.prototype.use ( value )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. Perform ? AddDisposableResource(disposableStack.[[DisposeCapability]], value, sync-dispose).
  ...

  AddDisposableResource ( disposeCapability, V, hint [, method ] )

  1. If method is not present then,
    a. If V is either null or undefined and hint is sync-dispose, then
      i. Return unused
    b. Let resource be ? CreateDisposableResource(V, hint).
  2. Else,
    ...
  3. Append resource to disposeCapability.[[DisposableResourceStack]].
  4. Return unused.

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
var disposed = [];
var resource1 = {
    [Symbol.dispose]() {
        disposed.push(this);
    }
};
var resource2 = {
    [Symbol.dispose]() {
        disposed.push(this);
    }
};
stack.use(resource1);
stack.use(resource2);
stack.dispose();
assert.sameValue(2, disposed.length);
assert.sameValue(disposed[0], resource2, 'Expected resource2 to be the first disposed resource');
assert.sameValue(disposed[1], resource1, 'Expected resource1 to be the second disposed resource');
