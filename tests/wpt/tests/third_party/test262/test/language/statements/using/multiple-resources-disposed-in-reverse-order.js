// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposeresources
description: Multiple resources are disposed in the reverse of the order in which they were added
info: |
  RS: Evaluation
    Block : { StatementList }

    ...
    5. Let blockValue be the result of evaluating StatementList.
    6. Set blockValue to DisposeResources(blockEnv.[[DisposeCapability]], blockValue).
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

var disposed = [];
var resource1 = { [Symbol.dispose]() { disposed.push(this); } };
var resource2 = { [Symbol.dispose]() { disposed.push(this); } };
var resource3 = { [Symbol.dispose]() { disposed.push(this); } };

{
  using _1 = resource1, _2 = resource2;
  using _3 = resource3;
}

assert.sameValue(disposed[0], resource3);
assert.sameValue(disposed[1], resource2);
assert.sameValue(disposed[2], resource1);
