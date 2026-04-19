// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test use() on a disposed stack.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  async function TestAsyncDisposableStackUseOnDisposedStack() {
    let stack = new AsyncDisposableStack();
    const disposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        return 42;
      }
    };
    await stack.disposeAsync();
    stack.use(disposable);
  };

  await assert.throwsAsync(
      ReferenceError, () => TestAsyncDisposableStackUseOnDisposedStack(),
      'Cannot add values to a disposed stack!');
});
