// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: An 'await using' implies an Await occurs if the statement is evaluated, even if all initializers are 'null' or 'undefined'
info: |
  RS: Evaluation
    AwaitUsingDeclaration : CoverAwaitExpressionAndAwaitUsingDeclarationHead BindingList ;

    1. Perform ? BindingEvaluation of BindingList with argument async-dispose.
    2. Return empty.

  RS: BindingEvaluation
    LexicalBinding : BindingIdentifier Initializer

    ...
    5. Return ? InitializeReferencedBinding(lhs, value, hint).

  InitializeReferencedBinding ( V, W )

  ...
  4. Return ? base.InitializeBinding(V.[[ReferencedName]], W).

  InitializeBinding ( N, V, hint )

  ...
  2. If hint is not normal, perform ? AddDisposableResource(envRec.[[DisposeCapability]], V, hint).
  ...

  AddDisposableResource ( disposeCapability, V, hint [, method ] )

  1. If method is not present then,
    a. If V is either null or undefined and hint is sync-dispose, then
      i. Return unused.
    b. Let resource be ? CreateDisposableResource(V, hint).
  ...

  CreateDisposableResource ( V, hint [ , method ] )

  1. If method is not present, then
    a. If V is either null or undefined, then
      i. Set V to undefined.
      ii. Set method to undefined.
    ...
  ...
  3. Return the DisposableResource Record { [[ResourceValue]]: V, [[Hint]]: hint, [[DisposeMethod]]: method }.

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
  var isRunningInSameMicrotask = true;
  var wasStartedInSameMicrotask = false;
  var didEvaluatePrecedingBlockStatementsInSameMicrotask = false;
  var didEvaluateFollowingBlockStatementsInSameMicrotask = false;
  var wasRunningInSameMicrotask = false;

  async function f() {
    wasStartedInSameMicrotask = isRunningInSameMicrotask;
    {
      didEvaluatePrecedingBlockStatementsInSameMicrotask = isRunningInSameMicrotask;
      await using _ = null;
      didEvaluateFollowingBlockStatementsInSameMicrotask = isRunningInSameMicrotask;
    }
    wasRunningInSameMicrotask = isRunningInSameMicrotask;
  }

  var p = f();
  isRunningInSameMicrotask = false;
  await p;

  assert.sameValue(wasStartedInSameMicrotask, true, 'Expected async function containing `await using` to start in the same microtask');
  assert.sameValue(didEvaluatePrecedingBlockStatementsInSameMicrotask, true, 'Expected block statements preceding `await using` to be evaluated in the same microtask');
  assert.sameValue(didEvaluateFollowingBlockStatementsInSameMicrotask, true, 'Expected block statements following `await using` to be evaluated in the same microtask');
  assert.sameValue(wasRunningInSameMicrotask, false, 'Expected statements following the block containing evaluated `await using` to evaluate in a different microtask');
});
