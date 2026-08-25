// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test defer() on disposed stack.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  async function TestAsyncDisposableStackDeferOnDisposedStack() {
    let stack = new AsyncDisposableStack();
    await stack.disposeAsync();
    stack.defer(() => {});
  };

  await assert.throwsAsync(
      ReferenceError, () => TestAsyncDisposableStackDeferOnDisposedStack(),
      'Cannot add values to a disposed stack!');
});
