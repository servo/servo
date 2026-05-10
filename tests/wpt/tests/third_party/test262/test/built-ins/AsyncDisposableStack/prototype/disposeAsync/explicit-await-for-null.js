// Copyright (C) 2026 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Awaits even when only null values are disposed.
info: |
  AsyncDisposableStack.prototype.disposeAsync ( )

  [...]
  6. Let result be DisposeResources(asyncDisposableStack.[[DisposeCapability]], NormalCompletion(undefined)).
  [...]

  DisposeResources ( disposeCapability, completion )

  1. Let needsAwait be false.
  2. Let hasAwaited be false.
  3. For each element resource of disposeCapability.[[DisposableResourceStack]], in reverse list order, do
     [...]
     f. Else,
        i. Assert: hint is async-dispose.
        ii. Set needsAwait to true.
        iii. NOTE: This can only indicate a case where either null or undefined was the initialized value of an await using declaration.
  4. If needsAwait is true and hasAwaited is false, then
     a. Perform ! Await(undefined).

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  var stack = new AsyncDisposableStack();
  var sequence = [];

  stack.use(null);

  await Promise.all([
    Promise.resolve()
      .then(() => 0)
      .then(() => { sequence.push('job 1'); }),
    stack.disposeAsync().then(() => { sequence.push('dispose'); }),
    Promise.resolve()
      .then(() => 0)
      .then(() => { sequence.push('job 2'); })
  ]);
  assert.compareArray(sequence, ['job 1', 'dispose', 'job 2']);
});
