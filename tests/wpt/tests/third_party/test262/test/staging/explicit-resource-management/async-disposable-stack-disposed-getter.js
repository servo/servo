// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test `disposed` accessor property of AsyncDisposableStack.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  // disposed should be true --------
  async function TestDisposableStackDisposedTrue() {
    let stack = new AsyncDisposableStack();
    const disposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        return 42;
      }
    };
    stack.use(disposable);
    stack.dispose();
    assert.sameValue(stack.disposed, true, 'disposed should be true');
  };

  TestDisposableStackDisposedTrue();

  // disposed should be false --------
  async function TestDisposableStackDisposedFalse() {
    let stack = new AsyncDisposableStack();
    const disposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        return 42;
      }
    };
    stack.use(disposable);
    assert.sameValue(stack.disposed, false, 'disposed should be false');
  };
  TestDisposableStackDisposedFalse();
});
