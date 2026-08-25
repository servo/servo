// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Fix async disposal from sync method returning a promise.
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let values = [];

  async function TestAsyncDisposalWithSyncMethodReturningAPromise() {
    let stack = new AsyncDisposableStack();
    const neverResolves = Promise.withResolvers().promise;
    stack.use({
      [Symbol.dispose]() {
        return neverResolves
      }
    });
    await stack.disposeAsync();
    values.push(42);

    await using x = {[Symbol.dispose]: () => neverResolves};
    values.push(43);
  };

  await TestAsyncDisposalWithSyncMethodReturningAPromise();

  assert.compareArray(values, [42, 43]);
});
