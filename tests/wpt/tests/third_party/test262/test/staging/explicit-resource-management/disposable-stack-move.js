// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test developer exposed DisposableStack protype method move.
includes: [compareArray.js]
features: [explicit-resource-management]
---*/

// move() method on disposed stack --------
function TestDisposableStackMoveOnDisposedStack() {
  let stack = new DisposableStack();
  stack.dispose();
  let newStack = stack.move();
};

assert.throws(
    ReferenceError, () => TestDisposableStackMoveOnDisposedStack(),
    'Cannot move elements from a disposed stack!');

// move() method --------
let valuesNormal = [];

(function TestDisposableStackMove() {
  let stack = new DisposableStack();
  const firstDisposable = {
    value: 1,
    [Symbol.dispose]() {
      valuesNormal.push(42);
    }
  };
  const secondDisposable = {
    value: 2,
    [Symbol.dispose]() {
      valuesNormal.push(43);
    }
  };
  stack.use(firstDisposable);
  stack.use(secondDisposable);
  let newStack = stack.move();
  newStack.dispose();
  // stack is already disposed, so the next line should do nothing.
  stack.dispose();
})();
assert.compareArray(valuesNormal, [43, 42]);

// Two stacks should not be the same--------
(function TestDisposableStackMoveNotSameObjects() {
  let stack = new DisposableStack();
  const firstDisposable = {
    value: 1,
    [Symbol.dispose]() {
      return 42;
    }
  };
  const secondDisposable = {
    value: 2,
    [Symbol.dispose]() {
      return 43;
    }
  };
  stack.use(firstDisposable);
  stack.use(secondDisposable);
  let newStack = stack.move();
  assert.notSameValue(stack, newStack);
})();
