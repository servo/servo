// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Call disposeAsync() on a disposed AsyncDisposableStack.
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let valuesNormal = [];

  async function TestAsyncDisposableStackUseDisposingTwice() {
    let stack = new AsyncDisposableStack();
    const firstDisposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        valuesNormal.push(42);
      }
    };
    const secondDisposable = {
      value: 2,
      [Symbol.asyncDispose]() {
        valuesNormal.push(43);
      }
    };
    stack.use(firstDisposable);
    stack.use(secondDisposable);
    let newStack = stack.move();
    await newStack.disposeAsync();
    assert.sameValue(newStack.disposed, true, 'disposed should be true');
    // stack is already disposed, so the next line should do nothing.
    await newStack.disposeAsync();
  };

  await TestAsyncDisposableStackUseDisposingTwice();

  assert.compareArray(valuesNormal, [43, 42]);
});
