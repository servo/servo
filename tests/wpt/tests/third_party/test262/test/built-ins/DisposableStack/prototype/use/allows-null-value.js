// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Does not throw when argument is 'null'
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
    ...
  ...

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
stack.use(null);
