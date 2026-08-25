// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.use
description: Puts value on the top of the dispose stack
info: |
  AsyncDisposableStack.prototype.use ( value )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  4. Perform ? AddDisposableResource(asyncDisposableStack.[[DisposeCapability]], value, async-dispose).
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

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  var stack = new AsyncDisposableStack();
  var disposed = [];
  var resource1 = {
      async [Symbol.asyncDispose]() {
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
  await stack.disposeAsync();
  assert.sameValue(2, disposed.length);
  assert.sameValue(disposed[0], resource2, 'Expected resource2 to be the first disposed resource');
  assert.sameValue(disposed[1], resource1, 'Expected resource1 to be the second disposed resource');
});
