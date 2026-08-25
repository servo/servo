// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Throws if initialized value's Symbol.asyncDispose property is null and Symbol.dispose is not present
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
  2. Else,
    ...
  3. Append resource to disposeCapability.[[DisposableResourceStack]].
  4. Return unused.

  CreateDisposableResource ( V, hint [ , method ] )

  1. If method is not present, then
    a. If V is either null or undefined, then
      ...
    b. Else,
      i. If V is not an Object, throw a TypeError exception.
      ii. Set method to ? GetDisposeMethod(V, hint).
      iii. If method is undefined, throw a TypeError exception.
  ...

  GetDisposeMethod ( V, hint )

  1. If hint is async-dispose, then
    a. Let method be ? GetMethod(V, @@asyncDispose).
    b. If method is undefined, then
      i. Set method to ? GetMethod(V, @@dispose).
  2. Else,
    a. Let method be ? GetMethod(V, @@dispose).
  3. Return method.

  GetMethod ( V, P )

  1. Let func be ? GetV(V, P).
  2. If func is either undefined or null, return undefined.
  3. ...

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  await assert.throwsAsync(TypeError, async function () {
      await using x = { [Symbol.asyncDispose]: null };
  });
});
