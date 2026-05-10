// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.defer
description: Adds a disposable resource to the stack
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
  var disposed = [];
  async function dispose1() { disposed.push(dispose1); }
  function dispose2() { disposed.push(dispose2); }
  stack.defer(dispose1);
  stack.defer(dispose2);
  await stack.disposeAsync();
  assert.sameValue(2, disposed.length);
  assert.sameValue(disposed[0], dispose2, 'Expected dispose2 to be the first onDisposeAsync invoked');
  assert.sameValue(disposed[1], dispose1, 'Expected dispose1 to be the second onDisposeAsync invoked');
});
