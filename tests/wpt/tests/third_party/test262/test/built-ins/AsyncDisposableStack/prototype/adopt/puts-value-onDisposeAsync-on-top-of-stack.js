// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.adopt
description: Adds a disposable resource to the stack
info: |
  AsyncDisposableStack.prototype.adopt ( value, onDisposeAsync )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  4. If IsCallable(onDisposeAsync) is false, throw a TypeError exception.
  5. Let closure be a new Abstract Closure with no parameters that captures value and onDisposeAsync and performs the following steps when called:
    a. Perform ? Call(onDisposeAsync, undefined, « value »).
  6. Let F be CreateBuiltinFunction(closure, 0, "", « »).
  7. Perform ? AddDisposableResource(asyncDisposableStack.[[DisposeCapability]], undefined, async-dispose, F).
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
  var resource1 = {};
  async function dispose1(res) { disposed.push([res, dispose1]); }
  var resource2 = {};
  function dispose2(res) { disposed.push([res, dispose2]); }
  stack.adopt(resource1, dispose1);
  stack.adopt(resource2, dispose2);
  await stack.disposeAsync();
  assert.sameValue(2, disposed.length);
  assert.sameValue(disposed[0][0], resource2, 'Expected resource2 to be the first disposed resource');
  assert.sameValue(disposed[0][1], dispose2, 'Expected dispose2 to be the first onDispose invoked');
  assert.sameValue(disposed[1][0], resource1, 'Expected resource1 to be the second disposed resource');
  assert.sameValue(disposed[1][1], dispose1, 'Expected dispose1 to be the second onDispose invoked');
});
