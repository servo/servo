// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Reads `[Symbol.dispose]` method if `[Symbol.asyncDispose]` is null
info: |
  RS: Evaluation
    AwaitUsingDeclaration : CoverAwaitExpressionAndAwaitUsingDeclarationHead BindingList ;

    1. Perform ? BindingEvaluation of BindingList with argument async-dispose.
    2. Return empty.

  RS: BindingEvaluation
    BindingList : BindingList , LexicalBinding

    1. Perform ? BindingEvaluation of BindingList with argument hint.
    2. Perform ? BindingEvaluation of LexicalBinding with argument hint.
    3. Return unused.

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
      i. Return unused
    b. Let resource be ? CreateDisposableResource(V, hint).
  ...

  CreateDisposableResource ( V, hint [ , method ] )

  1. If method is not present, then
    a. If V is either null or undefined, then
      i. Set V to undefined
      ii. Set method to undefined
    b. Else,
      i. If Type(V) is not Object, throw a TypeError exception.
      ii. Set method to ? GetDisposeMethod(V, hint).
      iii. If method is undefined, throw a TypeError exception.
  2. Else,
      a. ...
  3. Return the DisposableResource Record { [[ResourceValue]]: V, [[Hint]]: hint, [[DisposeMethod]]: method }.

  GetDisposeMethod ( V, hint )

  1. If hint is async-dispose, then
    a. Let method be ? GetMethod(V, @@asyncDispose).
    b. If method is undefined, then
      i. Set method to ? GetMethod(V, @@dispose).
  2. Else,
    a. Let method be ? GetMethod(V, @@dispose).
  3. Return method.

flags: [async]
includes: [asyncHelpers.js, deepEqual.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  var order = [];
  var resource = {
      get [Symbol.asyncDispose]() {
        order.push('Symbol.asyncDispose');
        return null;
      },
      get [Symbol.dispose]() {
          order.push('Symbol.dispose');
          return function() { };
      }
  };

  {
    await using _ = resource;
  }

  assert.deepEqual(order, ['Symbol.asyncDispose', 'Symbol.dispose'], 'Expected [Symbol.dispose] to have been read after [Symbol.asyncDispose] returns null');
});
