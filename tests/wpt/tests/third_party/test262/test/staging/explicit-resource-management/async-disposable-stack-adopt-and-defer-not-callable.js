// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed DisposableStack protype methods adopt() and defer().
features: [explicit-resource-management]
---*/

// adopt() method when onDispose is not callable--------
function TestAsyncDisposableStackAdoptWithNonCallableOnDispose() {
  let stack = new AsyncDisposableStack();
  stack.adopt(42, 43);
};
assert.throws(
    TypeError, () => TestAsyncDisposableStackAdoptWithNonCallableOnDispose(),
    'onDispose is not callable');

// defer() method when onDispose is not callable--------
function TestAsyncDisposableStackDeferWithNonCallableOnDispose() {
  let stack = new AsyncDisposableStack();
  stack.defer(42);
};
assert.throws(
    TypeError, () => TestAsyncDisposableStackDeferWithNonCallableOnDispose(),
    'onDispose is not callable');
