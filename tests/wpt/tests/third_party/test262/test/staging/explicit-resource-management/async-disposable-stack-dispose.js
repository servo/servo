// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed AsyncDisposableStack protype methods disposeAsync().
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let valuesNormal = [];

  async function TestAsyncDisposableStackUse() {
    let stack = new AsyncDisposableStack();
    const disposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        valuesNormal.push(42);
      }
    };
    stack.use(disposable);
    await stack.disposeAsync();
  };

  await TestAsyncDisposableStackUse();
  assert.compareArray(valuesNormal, [42]);
});
