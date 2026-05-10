// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.defer
description: Adds an onDisposeAsync callback to the stack
info: |
  AsyncDisposableStack.prototype.defer ( onDisposeAsync )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  4. If IsCallable(onDisposeAsync) is false, throw a TypeError exception.
  5. Perform ? AddDisposableResource(asyncDisposableStack.[[DisposeCapability]], undefined, async-dispose, onDisposeAsync).
  ...

  AddDisposableResource ( disposeCapability, V, hint [, method ] )

  1. If method is not present then,
    ...
  2. Else,
    a. Assert: V is undefined.
    b. Let resource be ? CreateDisposableResource(undefined, hint, method).
  3. Append resource to disposeCapability.[[DisposableResourceStack]].
  4. Return unused.

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  var stack = new AsyncDisposableStack();
  var disposed = false;
  stack.defer(async () => { disposed = true });
  await stack.disposeAsync();
  assert.sameValue(disposed, true, 'Expected callback to have been called');
});
