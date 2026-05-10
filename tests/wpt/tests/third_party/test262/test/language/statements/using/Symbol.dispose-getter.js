// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Invokes [Symbol.dispose] getter
info: |
  RS: Evaluation
    UsingDeclaration : using BindingList ;

    1. Perform ? BindingEvaluation of BindingList with argument sync-dispose.
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
      i. Set V to undefined.
      ii. Set method to undefined.
    b. Else,
      i. If V is not an Object, throw a TypeError exception.
      ii. Set method to ? GetDisposeMethod(V, hint).
      iii. If method is undefined, throw a TypeError exception.
  2. Else,
    ...
  3. Return the DisposableResource Record { [[ResourceValue]]: V, [[Hint]]: hint, [[DisposeMethod]]: method }.

  GetDisposeMethod ( V, hint )

  1. If hint is async-dispose, then
    ...
  2. Else,
    a. Let method be ? GetMethod(V, @@dispose).
  3. Return method.

  GetMethod ( V, P )

  1. Let func be ? GetV(V, P).
  2. If func is either undefined or null, return undefined.
  3. If IsCallable(func) is false, throw a TypeError exception.
  4. Return func.

features: [explicit-resource-management]
---*/

var resource = {
  disposed: false,
  get [Symbol.dispose]() {
    return function() {
      this.disposed = true;
    };
  }
};

{
  using _ = resource;
}

assert.sameValue(resource.disposed, true, 'Expected resource to have been disposed');
