// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposeresources
description: >
  Rethrows an error as-is if it is the only error thrown during evaluation of subsequent statements following 'await using'
  or from disposal.
info: |
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

asyncTest(async function() {
  class MyError extends Error {}
  await assert.throwsAsync(MyError, async function () {
    await using _1 = { async [Symbol.asyncDispose]() { throw new MyError(); } };
    await using _2 = { [Symbol.dispose]() { } };
  });

  await assert.throwsAsync(MyError, async function () {
    await using _1 = { async [Symbol.asyncDispose]() { } };
    await using _2 = { [Symbol.dispose]() { throw new MyError(); } };
  });

  await assert.throwsAsync(MyError, async function () {
    await using _1 = { async [Symbol.asyncDispose]() { } };
    await using _2 = { [Symbol.dispose]() { } };
    throw new MyError();
  });
});
