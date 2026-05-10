// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed AsyncDisposableStack protype methods adopt().
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let valuesNormal = [];

  async function TestAsyncDisposableStackAdopt() {
    let stack = new AsyncDisposableStack();
    stack.adopt(42, function(v) {
      valuesNormal.push(v)
    });
    const disposable = {
      value: 1,
      [Symbol.asyncDispose]() {
        valuesNormal.push(43);
      }
    };
    stack.use(disposable);
    stack.adopt(44, function(v) {
      valuesNormal.push(v)
    });
    await stack.disposeAsync();
  };

  await TestAsyncDisposableStackAdopt();
  assert.compareArray(valuesNormal, [44, 43, 42]);
});
