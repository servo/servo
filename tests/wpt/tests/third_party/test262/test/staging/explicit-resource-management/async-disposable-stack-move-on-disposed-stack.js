// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test move() on a disposed-stack.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

// move() method on disposed stack --------
asyncTest(async function() {
  async function TestAsyncDisposableStackMoveOnDisposedStack() {
    let stack = new AsyncDisposableStack();
    await stack.disposeAsync();
    let newStack = stack.move();
  };

  await assert.throwsAsync(
      ReferenceError, () => TestAsyncDisposableStackMoveOnDisposedStack(),
      'Cannot move elements from a disposed stack!');
});
