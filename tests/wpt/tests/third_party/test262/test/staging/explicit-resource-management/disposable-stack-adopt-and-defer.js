// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed DisposableStack protype methods adopt() and defer().
includes: [compareArray.js]
features: [explicit-resource-management]
---*/

// adopt() method on disposed stack --------
function TestDisposableStackAdoptOnDisposedStack() {
    let stack = new DisposableStack();
    stack.dispose();
    stack.adopt(42, function(v) {return v});
  };
  assert.throws(
      ReferenceError, () => TestDisposableStackUseOnDisposedStack(),
      'Cannot add values to a disposed stack!');

// adopt() method when onDispose is not callable--------
function TestDisposableStackAdoptWithNonCallableOnDispose() {
  let stack = new DisposableStack();
  stack.adopt(42, 43);
};
assert.throws(
    TypeError, () => TestDisposableStackAdoptWithNonCallableOnDispose(),
    'onDispose is not callable');

// adopt() method --------
let valuesNormal = [];

(function TestDisposableStackAdopt() {
  let stack = new DisposableStack();
  stack.adopt(42, function(v) {valuesNormal.push(v)});
  const disposable = {
    value: 1,
    [Symbol.dispose]() {
      valuesNormal.push(43);
    }
  };
  stack.use(disposable);
  stack.adopt(44, function(v) {valuesNormal.push(v)});
  stack.dispose();
})();
assert.compareArray(valuesNormal, [44, 43, 42]);

// defer() method on disposed stack --------
function TestDisposableStackDeferOnDisposedStack() {
  let stack = new DisposableStack();
  stack.dispose();
  stack.defer(() => console.log(42));
};

assert.throws(
    ReferenceError, () => TestDisposableStackDeferOnDisposedStack(),
    'Cannot add values to a disposed stack!');

// defer() method when onDispose is not callable--------
function TestDisposableStackDeferWithNonCallableOnDispose() {
  let stack = new DisposableStack();
  stack.defer(42);
};
assert.throws(
    TypeError, () => TestDisposableStackDeferWithNonCallableOnDispose(),
    'onDispose is not callable');

// defer() method --------
let deferValuesNormal = [];

(function TestDisposableStackAdopt() {
  let stack = new DisposableStack();
  stack.defer(() => deferValuesNormal.push(42));
  const disposable = {
    value: 1,
    [Symbol.dispose]() {
      deferValuesNormal.push(43);
    }
  };
  stack.use(disposable);
  stack.defer(() => deferValuesNormal.push(44));
  stack.dispose();
})();
assert.compareArray(deferValuesNormal, [44, 43, 42]);
