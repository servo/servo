// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
description: Initialized value is disposed at end of each iteration of ForOfStatement
info: |
  ForIn/OfBodyEvaluation ( lhs, stmt, iteratorRecord, iterationKind, lhsKind, labelSet [ , iteratorKind ] )

  1. If iteratorKind is not present, set iteratorKind to sync.
  2. Let oldEnv be the running execution context's LexicalEnvironment.
  3. Let V be undefined.
  4. If IsAwaitUsingDeclaration of lhs is true, then
    a. Let hint be async-dispose.
  5. Else, if IsUsingDeclaration of lhs is true, then
    a. Let hint be sync-dispose.
  6. Else,
    a. Let hint be normal.
  7. Let destructuring be IsDestructuring of lhs.
  8. If destructuring is true and if lhsKind is assignment, then
    a. Assert: lhs is a LeftHandSideExpression.
    b. Let assignmentPattern be the AssignmentPattern that is covered by lhs.
  9. Repeat,
    ...
    j. Let result be Completion(Evaluation of stmt).
    k. If iterationEnv is not undefined, then
      i. Set result to Completion(DisposeResources(iterationEnv.[[DisposeCapability]], result)).
    ...

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

  var wasDisposedBeforeBody = false;
  var wasDisposedBeforeIteration = false;
  var wasDisposedAfterIteration = false;

  function * g() {
    wasDisposedBeforeIteration = resource.disposed;
    yield resource;
    wasDisposedAfterIteration = resource.disposed;
  }

  for (await using _ of g()) {
    wasDisposedBeforeBody = resource.disposed;
  }

  assert.sameValue(wasDisposedBeforeIteration, false, 'Expected resource to not been disposed before the for-of loop has received a value');
  assert.sameValue(wasDisposedBeforeBody, false, 'Expected resource to not been disposed while for-of loop is still iterating');
  assert.sameValue(wasDisposedAfterIteration, true, 'Expected resource to have been disposed after the for-of loop advanced to the next value');
});
