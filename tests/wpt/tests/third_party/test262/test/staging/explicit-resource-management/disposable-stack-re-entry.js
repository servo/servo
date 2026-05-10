// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Re-entry to a disposable stack should do nothing.
includes: [compareArray.js]
features: [explicit-resource-management]
---*/

let values = [];

(function TestDisposableStackReEntry() {
  let stack = new DisposableStack();
  stack.use({
    [Symbol.dispose]() {
      values.push(42);
      stack.dispose();
    }
  });
  stack.dispose();
})();
assert.compareArray(values, [42]);
