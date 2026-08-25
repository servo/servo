// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Does not re-invoke disposal on resources after stack has already been disposed.
info: |
  AsyncDisposableStack.prototype.disposeAsync ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, return undefined.
  4. Set asyncDisposableStack.[[AsyncDisposableState]] to disposed.
  5. Return DisposeResources(asyncDisposableStack.[[DisposeCapability]], NormalCompletion(undefined)).

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
  var useCount = 0;
  var adoptCount = 0;
  var deferCount = 0;
  stack.use({ async [Symbol.asyncDispose]() { useCount++; } });
  stack.adopt({}, _ => { adoptCount++; });
  stack.defer(() => { deferCount++; });
  var p1 = stack.disposeAsync();
  var p2 = stack.disposeAsync();
  await Promise.all([p1, p2]);
  assert.sameValue(useCount, 1);
  assert.sameValue(adoptCount, 1);
  assert.sameValue(deferCount, 1);
});
