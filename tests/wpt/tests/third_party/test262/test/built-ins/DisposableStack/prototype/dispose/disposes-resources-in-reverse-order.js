// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.dispose
description: Added resources are disposed in reverse order
info: |
  UsingDeclaration : using BindingList ;

  1. Perform ? BindingEvaluation of BindingList with argument sync-dispose.
  2. Return empty.

  BindingList : BindingList , LexicalBinding

  1. Perform ? BindingEvaluation of BindingList with argument hint.
  2. Perform ? BindingEvaluation of LexicalBinding with argument hint.
  3. Return unused.

  LexicalBinding : BindingIdentifier Initializer

  1. Let bindingId be StringValue of BindingIdentifier.
  2. Let lhs be ? ResolveBinding(bindingId).
  3. If IsAnonymousFunctionDefinition(Initializer) is true, then
    a. Let value be NamedEvaluation of Initializer with argument bindingId.
  4. Else,
    a. Let rhs be the result of evaluating Initializer.
    b. Let value be ? GetValue(rhs).
  5. Return ? InitializeReferencedBinding(lhs, value, hint).

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

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
var disposed = [];
var resource1 = { [Symbol.dispose]() { disposed.push(resource1); } };
var resource2 = {};
function dispose2(res) { disposed.push(res); }
function dispose3() { disposed.push(dispose3); }
stack.use(resource1);
stack.adopt(resource2, dispose2);
stack.defer(dispose3);
stack.dispose();
assert.sameValue(disposed[0], dispose3);
assert.sameValue(disposed[1], resource2);
assert.sameValue(disposed[2], resource1);
