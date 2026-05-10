// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.adopt
description: Adds a disposable resource to the stack
info: |
  DisposableStack.prototype.adopt ( value, onDispose )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. If IsCallable(onDispose) is false, throw a TypeError exception.
  5. Let closure be a new Abstract Closure with no parameters that captures value and onDispose and performs the following steps when called:
    a. Perform ? Call(onDispose, undefined, « value »).
  6. Let F be CreateBuiltinFunction(closure, 0, "", « »).
  7. Perform ? AddDisposableResource(disposableStack.[[DisposeCapability]], undefined, sync-dispose, F).
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
var resource = { disposed: false };
stack.adopt(resource, r => { r.disposed = true });
stack.dispose();
assert.sameValue(resource.disposed, true, 'Expected resource to have been disposed');
