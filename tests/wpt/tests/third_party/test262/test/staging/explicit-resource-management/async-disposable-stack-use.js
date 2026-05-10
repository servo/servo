// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed AsyncDisposableStack protype methods use().
features: [explicit-resource-management]
---*/

// use() method on a non object --------
function TestAsyncDisposableStackUseWithNonObject() {
  let stack = new AsyncDisposableStack();
  stack.use(42);
};
assert.throws(
    TypeError, () => TestAsyncDisposableStackUseWithNonObject(),
    'use() is called on non-object');

// use() method with null [symbol.asyncDispose] --------
function TestAsyncDisposableStackUseWithNullDispose() {
  let stack = new AsyncDisposableStack();
  const disposable = {
    value: 1,
    [Symbol.asyncDispose]: null,
  };
  stack.use(disposable);
};
assert.throws(
    TypeError, () => TestAsyncDisposableStackUseWithNullDispose(),
    'symbol.asyncDispose is null');

// use() method with undefined [symbol.asyncDispose] --------
function TestAsyncDisposableStackUseWithUndefinedDispose() {
  let stack = new AsyncDisposableStack();
  const disposable = {
    value: 1,
    [Symbol.asyncDispose]: undefined,
  };
  stack.use(disposable);
};
assert.throws(
    TypeError, () => TestAsyncDisposableStackUseWithUndefinedDispose(),
    'symbol.asyncDispose is undefined');

// use() method when [symbol.asyncDispose] is not callable--------
function TestAsyncDisposableStackUseWithNonCallableDispose() {
  let stack = new AsyncDisposableStack();
  const disposable = {
    value: 1,
    [Symbol.asyncDispose]: 42,
  };
  stack.use(disposable);
};
assert.throws(
    TypeError, () => TestAsyncDisposableStackUseWithNonCallableDispose(),
    'symbol.asyncDispose is not callable');
