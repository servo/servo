// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed AsyncDisposableStack protype method defer().
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let deferValuesNormal = [];

  async function TestAsyncDisposableStackDefer() {
    let stack = new AsyncDisposableStack();
    stack.defer(() => deferValuesNormal.push(42));
    const disposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        deferValuesNormal.push(43);
      }
    };
    stack.use(disposable);
    stack.defer(() => deferValuesNormal.push(44));
    await stack.disposeAsync();
  };

  await TestAsyncDisposableStackDefer();
  assert.compareArray(deferValuesNormal, [44, 43, 42]);
});
