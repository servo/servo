// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test `disposed` accessor property of DisposableStack.
features: [explicit-resource-management]
---*/

// disposed should be true --------
(function TestDisposableStackDisposedTrue() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]() {
      return 42;
    }
  };
  stack.use(disposable);
  stack.dispose();
  assert.sameValue(stack.disposed, true, 'disposed should be true');
})();

// disposed should be false --------
(function TestDisposableStackDisposedFalse() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]() {
      return 42;
    }
  };
  stack.use(disposable);
  assert.sameValue(stack.disposed, false, 'disposed should be false');
})();
