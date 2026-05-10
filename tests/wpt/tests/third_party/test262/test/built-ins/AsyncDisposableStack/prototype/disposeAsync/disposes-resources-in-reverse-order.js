// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Added resources are disposed in reverse order
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
  var stack = new AsyncDisposableStack();
  var disposed = [];
  var resource1 = { async [Symbol.asyncDispose]() { disposed.push(this); } };
  var resource2 = { [Symbol.dispose]() { disposed.push(this); } };
  var resource3 = {};
  async function dispose3(res) { disposed.push(res); }
  var resource4 = {};
  function dispose4(res) { disposed.push(res); }
  async function dispose5() { disposed.push(dispose5); }
  function dispose6() { disposed.push(dispose6); }
  stack.use(resource1);
  stack.use(resource2);
  stack.adopt(resource3, dispose3);
  stack.adopt(resource4, dispose4);
  stack.defer(dispose5);
  stack.defer(dispose6);
  await stack.disposeAsync();
  assert.compareArray(disposed, [dispose6, dispose5, resource4, resource3, resource2, resource1])
});
