// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test adopt() on a disposed stack.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  async function TestAsyncDisposableStackAdoptOnDisposedStack() {
    let stack = new AsyncDisposableStack();
    await stack.disposeAsync();
    stack.adopt(42, function(v) {
      return v
    });
  };
  await assert.throwsAsync(
      ReferenceError, () => TestAsyncDisposableStackAdoptOnDisposedStack(),
      'Cannot add values to a disposed stack!');
});
