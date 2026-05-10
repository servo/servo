// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Exposed AsyncDisposableStack protype methods disposeAsync() throws.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  async function TestAsyncDisposableStackUseDisposeMethodThrows() {
    {
      let stack = new AsyncDisposableStack();
      const disposable = {
        value: 1,
        [Symbol.asyncDispose]() {
          throw new Test262Error('Symbol.asyncDispose is throwing!');
        }
      };
      stack.use(disposable);
      await stack.disposeAsync();
    }
  };
  await assert.throwsAsync(
      Test262Error, () => TestAsyncDisposableStackUseDisposeMethodThrows(),
      'Symbol.asyncDisposeispose is throwing!');
});
