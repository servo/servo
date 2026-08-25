// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Disposal succeeds even if [Symbol.disposeAsync] does not return a Promise.
info: |
  AsyncDisposableStack.prototype.disposeAsync ( )

  1. Let asyncDisposableStack be the this value.
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  3. If asyncDisposableStack does not have an [[AsyncDisposableState]] internal slot, then
    a. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
    b. Return promiseCapability.[[Promise]].
  4. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, then
    a. Perform ! Call(promiseCapability.[[Resolve]], undefined, « undefined »).
    b. Return promiseCapability.[[Promise]].
  5. Set asyncDisposableStack.[[AsyncDisposableState]] to disposed.
  6. Let result be DisposeResources(asyncDisposableStack.[[DisposeCapability]], NormalCompletion(undefined)).
  7. IfAbruptRejectPromise(result, promiseCapability).
  8. Perform ! Call(promiseCapability.[[Resolve]], undefined, « result »).
  9. Return promiseCapability.[[Promise]].

  DisposeResources ( disposeCapability, completion )

  1. For each resource of disposeCapability.[[DisposableResourceStack]], in reverse list order, do
    a. Let result be Dispose(resource.[[ResourceValue]], resource.[[Hint]], resource.[[DisposeMethod]]).
    b. If result.[[Type]] is throw, then
      i. If completion.[[Type]] is throw, then
        1. Set result to result.[[Value]].
        2. Let suppressed be completion.[[Value]].
        3. Let error be a newly created SuppressedError object.
        4. Perform ! CreateNonEnumerableDataPropertyOrThrow(error, "error", result).
        5. Perform ! CreateNonEnumerableDataPropertyOrThrow(error, "suppressed", suppressed).
        6. Set completion to ThrowCompletion(error).
      ii. Else,
        1. Set completion to result.
  2. Return completion.

  Dispose ( V, hint, method )

  1. If method is undefined, let result be undefined.
  2. Else, let result be ? Call(method, V).
  3. If hint is async-dispose, then
    a. Perform ? Await(result).
  4. Return undefined.

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  var resource = {
    disposed: false,
    [Symbol.asyncDispose]() {
      this.disposed = true;
    }
  };

  var stack = new AsyncDisposableStack();
  stack.use(resource);
  await stack.disposeAsync();

  assert.sameValue(resource.disposed, true, 'Expected resource to have been disposed');
});
