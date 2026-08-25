// Copyright (C) 2026 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Does not await when the stack is empty.
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
  4. If needsAwait is true and hasAwaited is false, then
     a. Perform ! Await(undefined).

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  var stack = new AsyncDisposableStack();
  var sequence = [];

  await Promise.all([
    Promise.resolve()
      .then(() => { sequence.push('job 1'); }),
    stack.disposeAsync().then(() => { sequence.push('dispose'); }),
    Promise.resolve()
      .then(() => { sequence.push('job 2'); })
  ]);
  assert.compareArray(sequence, ['job 1', 'dispose', 'job 2']);
});
