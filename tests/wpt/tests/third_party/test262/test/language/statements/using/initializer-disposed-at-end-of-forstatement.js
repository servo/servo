// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-forloopevaluation
description: Initialized value is disposed at end of ForStatement
info: |
  RS: ForLoopEvaluation
    ForStatement : for ( LexicalDeclaration Expressionopt ; Expressionopt ) Statement

    ...
    12. Let bodyResult be Completion(ForBodyEvaluation(test, increment, Statement, perIterationLets, labelSet)).
    13. Set bodyResult to Completion(DisposeResources(loopEnv.[[DisposeCapability]], bodyResult)).
    14. Assert: If bodyResult.[[Type]] is normal, then bodyResult.[[Value]] is not empty.
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
    a. ...
  4. Return undefined.

features: [explicit-resource-management]
---*/

var resource = {
    disposed: false,
    [Symbol.dispose]() {
        this.disposed = true;
    }
};

var i = 0;
var wasDisposedInForStatement;
for (using _ = resource; i < 1; i++) {
  wasDisposedInForStatement = resource.disposed;
}

assert.sameValue(wasDisposedInForStatement, false, 'Expected resource to not been disposed while for loop is still iterating');
assert.sameValue(resource.disposed, true, 'Expected resource to have been disposed');
