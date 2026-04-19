// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.dispose
description: Throws multiple errors from a disposal nested in one or more SuppressedError instances.
info: |
  DisposableStack.prototype.dispose ( )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, return undefined.
  4. Set disposableStack.[[DisposableState]] to disposed.
  5. Return DisposeResources(disposableStack.[[DisposeCapability]], NormalCompletion(undefined)).

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

class MyError extends Error {}
var error1 = new MyError();
var error2 = new MyError();
var error3 = new MyError();
var stack = new DisposableStack();
stack.defer(function () { throw error1; });
stack.defer(function () { throw error2; });
stack.defer(function () { throw error3; });
try {
  stack.dispose();
  assert(false, 'Expected stack.dispose() to have thrown an error.');
}
catch (e) {
  assert(e instanceof SuppressedError, "Expected stack.dispose() to have thrown a SuppressedError");
  assert.sameValue(e.error, error1, "Expected the outermost suppressing error to have been 'error1'");
  assert(e.suppressed instanceof SuppressedError, "Expected the outermost suppressed error to have been a SuppressedError");
  assert.sameValue(e.suppressed.error, error2, "Expected the innermost suppressing error to have been 'error2'");
  assert.sameValue(e.suppressed.suppressed, error3, "Expected the innermost suppressed error to have been 'error3'");
}
