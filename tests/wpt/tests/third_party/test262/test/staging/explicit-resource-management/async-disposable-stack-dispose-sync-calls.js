// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Call disposeAsync() twice without await.
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let valuesNormal = [];

  async function TestAsyncDisposableStackUseDisposingTwiceWithoutAwait() {
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
    stack.disposeAsync();
    assert.sameValue(stack.disposed, true, 'disposed should be true');
    stack.disposeAsync();
  };

  await TestAsyncDisposableStackUseDisposingTwiceWithoutAwait();

  assert.compareArray(valuesNormal, [43, 42]);
});
