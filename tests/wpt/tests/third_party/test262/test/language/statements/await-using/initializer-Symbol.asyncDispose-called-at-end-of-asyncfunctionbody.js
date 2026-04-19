// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncblockstart
description: Initialized value is disposed at end of AsyncFunctionBody
info: |
  AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )

  1. Assert: promiseCapability is a PromiseCapability Record.
  2. Let runningContext be the running execution context.
  3. Let closure be a new Abstract Closure with no parameters that captures promiseCapability and asyncBody and performs the following steps when called:
    a. Let acAsyncContext be the running execution context.
    b. Let result be Completion(Evaluation of asyncBody).
    c. Assert: If we return here, the async function either threw an exception or performed an implicit or explicit return; all awaiting is done.
    d. Remove acAsyncContext from the execution context stack and restore the execution context that is at the top of the execution context stack as the running execution context.
    e. Let env be acAsyncContext's LexicalEnvironment.
    f. Set result to DisposeResources(env.[[DisposeCapability]], result).
    g. If result.[[Type]] is normal, then
      i. Perform ! Call(promiseCapability.[[Resolve]], undefined, « undefined »).
    h. Else if result.[[Type]] is return, then
      i. Perform ! Call(promiseCapability.[[Resolve]], undefined, « result.[[Value]] »).
    i. Else,
      i. Assert: result.[[Type]] is throw.
      ii. Perform ! Call(promiseCapability.[[Reject]], undefined, « result.[[Value]] »).
    j. Return unused.
  4. Set the code evaluation state of asyncContext such that when evaluation is resumed for that execution context, closure will be called with no arguments.
  5. Push asyncContext onto the execution context stack; asyncContext is now the running execution context.
  6. Resume the suspended evaluation of asyncContext. Let result be the value returned by the resumed computation.
  7. Assert: When we return here, asyncContext has already been removed from the execution context stack and runningContext is the currently running execution context.
  8. Assert: result is a normal completion with a value of unused. The possible sources of this value are Await or, if the async function doesn't await anything, step 3.h above.
  9. Return unused.

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
      async [Symbol.asyncDispose]() {
          this.disposed = true;
      }
  };

  var releaseF1;
  var suspendFPromise1 = new Promise(function (resolve) { releaseF1 = resolve; });

  var releaseBody;
  var suspendBodyPromise = new Promise(function (resolve) { releaseBody = resolve; });

  var releaseF2;
  var suspendFPromise2 = new Promise(function (resolve) { releaseF2 = resolve; });

  async function f() {
      await using _ = resource;
      await suspendFPromise1;
      releaseBody();
      await suspendFPromise2;
  }

  var resultPromise = f();

  var wasDisposedWhileSuspended1 = resource.disposed;

  releaseF1();
  await suspendBodyPromise;

  var wasDisposedWhileSuspended2 = resource.disposed;

  releaseF2();
  await resultPromise;

  var isDisposedAfterCompleted = resource.disposed;

  assert.sameValue(wasDisposedWhileSuspended1, false, 'Expected resource to not have been disposed while async function is suspended during await');
  assert.sameValue(wasDisposedWhileSuspended2, false, 'Expected resource to not have been disposed while async function is suspended during await');
  assert.sameValue(isDisposedAfterCompleted, true, 'Expected resource to have been disposed after async function completed');
});
