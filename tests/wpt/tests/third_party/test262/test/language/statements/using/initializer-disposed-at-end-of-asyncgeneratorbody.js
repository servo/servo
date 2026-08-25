// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgeneratorstart
description: Initialized value is disposed at end of AsyncGeneratorBody
info: |
  AsyncGeneratorStart ( generator, generatorBody )

  1. Assert: generator.[[AsyncGeneratorState]] is undefined.
  2. Let genContext be the running execution context.
  3. Set the Generator component of genContext to generator.
  4. Let closure be a new Abstract Closure with no parameters that captures generatorBody and performs the following steps when called:
    a. Let acGenContext be the running execution context.
    b. Let acGenerator be the Generator component of acGenContext.
    c. If generatorBody is a Parse Node, then
      i. Let result be Completion(Evaluation of generatorBody).
    d. Else,
      i. Assert: generatorBody is an Abstract Closure with no parameters.
      ii. Let result be Completion(generatorBody()).
    e. Assert: If we return here, the async generator either threw an exception or performed either an implicit or explicit return.
    f. Remove acGenContext from the execution context stack and restore the execution context that is at the top of the execution context stack as the running execution context.
    g. Set acGenerator.[[AsyncGeneratorState]] to completed.
    h. Let env be genContext's LexicalEnvironment.
    i. If env is not undefined, then
      i. Assert: env is a Declarative Environment Record
      ii. Set result to DisposeResources(env.[[DisposeCapability]], result).
    h. If result.[[Type]] is normal, set result to NormalCompletion(undefined).
    i. If result.[[Type]] is return, set result to NormalCompletion(result.[[Value]]).
    j. Perform AsyncGeneratorCompleteStep(acGenerator, result, true).
    k. Perform AsyncGeneratorDrainQueue(acGenerator).
    l. Return undefined.
  5. Set the code evaluation state of genContext such that when evaluation is resumed for that execution context, closure will be called with no arguments.
  6. Set generator.[[AsyncGeneratorContext]] to genContext.
  7. Set generator.[[AsyncGeneratorState]] to suspendedStart.
  8. Set generator.[[AsyncGeneratorQueue]] to a new empty List.
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
    a. ...
  4. Return undefined.

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {

  var resource = {
      disposed: false,
      [Symbol.dispose]() {
          this.disposed = true;
      }
  };

  var releaseF;
  var suspendFPromise = new Promise(function (resolve) { releaseF = resolve; });

  async function * f() {
      using _ = resource;
      yield;
      await suspendFPromise;
  }

  var g = f();

  var wasDisposedBeforeAsyncGeneratorStarted = resource.disposed;

  await g.next();

  var wasDisposedWhileSuspendedForYield = resource.disposed;

  var nextPromise = g.next();

  var wasDisposedWhileSuspendedForAwait = resource.disposed;

  releaseF();
  await nextPromise;

  var isDisposedAfterCompleted = resource.disposed;

  assert.sameValue(wasDisposedBeforeAsyncGeneratorStarted, false, 'Expected resource to not have been disposed prior to async generator start');
  assert.sameValue(wasDisposedWhileSuspendedForYield, false, 'Expected resource to not have been disposed while async generator function is suspended for yield');
  assert.sameValue(wasDisposedWhileSuspendedForAwait, false, 'Expected resource to not have been disposed while async generator function is suspended during await');
  assert.sameValue(isDisposedAfterCompleted, true, 'Expected resource to have been disposed after async generator function completed');
});
