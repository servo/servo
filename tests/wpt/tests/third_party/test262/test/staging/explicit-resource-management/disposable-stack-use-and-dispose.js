// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Test developer exposed DisposableStack protype methods use() and dispose().
includes: [compareArray.js]
features: [explicit-resource-management]
---*/

// use() method on null should not throw --------
let valuesWithNull = [];

(function TestDisposableStackUsewithNull() {
  let stack = new DisposableStack();
  valuesWithNull.push(42);
  stack.use(null);
  valuesWithNull.push(43);
})();
assert.compareArray(valuesWithNull, [42, 43]);

// use() method on undefined should not throw --------
let valuesWithUndefined = [];

(function TestDisposableStackUseWithUndefined() {
  let stack = new DisposableStack();
  valuesWithUndefined.push(42);
  stack.use(undefined);
  valuesWithUndefined.push(43);
})();
assert.compareArray(valuesWithUndefined, [42, 43]);

// use() method on a non object --------
function TestDisposableStackUseWithNonObject() {
  let stack = new DisposableStack();
  stack.use(42);
};
assert.throws(
    TypeError, () => TestDisposableStackUseWithNonObject(),
    'use() is called on non-object');

// use() method on normal value --------
let valuesNormal = [];

(function TestDisposableStackUse() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]() {
      valuesNormal.push(42);
    }
  };
  stack.use(disposable);
  stack.dispose();
})();
assert.compareArray(valuesNormal, [42]);

// use() method with null [symbol.dispose] --------
function TestDisposableStackUseWithNullDispose() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]: null,
  };
  stack.use(disposable);
};
assert.throws(
    TypeError, () => TestDisposableStackUseWithNullDispose(),
    'symbol.dispose is null');

// use() method with undefined [symbol.dispose] --------
function TestDisposableStackUseWithUndefinedDispose() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]: undefined,
  };
  stack.use(disposable);
};
assert.throws(
    TypeError, () => TestDisposableStackUseWithUndefinedDispose(),
    'symbol.dispose is undefined');

// use() method when [symbol.dispose] is not callable--------
function TestDisposableStackUseWithNonCallableDispose() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]: 42,
  };
  stack.use(disposable);
};
assert.throws(
    TypeError, () => TestDisposableStackUseWithNonCallableDispose(),
    'symbol.dispose is not callable');

// disposing a disposed stack should not throw --------
let valuesWithTwiceDisposal = [];

(function TestDisposableStackUseDisposingTwice() {
  let stack = new DisposableStack();
  const disposable = {
    value: 1,
    [Symbol.dispose]() {
      valuesWithTwiceDisposal.push(42);
    }
  };
  stack.use(disposable);
  stack.dispose();
  stack.dispose();
})();
assert.compareArray(valuesWithTwiceDisposal, [42]);

// use() method on disposed stack --------
function TestDisposableStackUseOnDisposedStack() {
  let stack = new DisposableStack();
  stack.dispose();
  stack.use(disposable);
};

assert.throws(
    ReferenceError, () => TestDisposableStackUseOnDisposedStack(),
    'Cannot add values to a disposed stack!');

// use() method with error inside disposal method --------
function TestDisposableStackUseDisposeMethodThrows() {
  {
    let stack = new DisposableStack();
    const disposable = {
      value: 1,
      [Symbol.dispose]() {
        throw new Test262Error('Symbol.dispose is throwing!');
      }
    };
    stack.use(disposable);
    stack.dispose();
  }
};
assert.throws(
    Test262Error, () => TestDisposableStackUseDisposeMethodThrows(),
    'Symbol.dispose is throwing!');

// use() method with suppressed error of two disposal method errors--------
let firstDisposeError =
    new Test262Error('The first Symbol.dispose is throwing!');
let secondDisposeError =
    new Test262Error('The second Symbol.dispose is throwing!');

function TestDisposableStackUseTwoDisposeMethodsThrow() {
  {
    let stack = new DisposableStack();
    const firstDisposable = {
      value: 1,
      [Symbol.dispose]() {
        throw firstDisposeError;
      }
    };
    const secondDisposable = {
      value: 1,
      [Symbol.dispose]() {
        throw secondDisposeError;
      }
    };
    stack.use(firstDisposable);
    stack.use(secondDisposable);
    stack.dispose();
  }
};

assert.throws(
    SuppressedError, () => TestDisposableStackUseTwoDisposeMethodsThrow(),
    'An error was suppressed during disposal');

function RunTestDisposableStackUseTwoDisposeMethodsThrow() {
  try {
    TestDisposableStackUseTwoDisposeMethodsThrow();
  } catch (error) {
    assert(
        error instanceof SuppressedError,
        'error is an instanceof SuppressedError');
    assert.sameValue(error.error, firstDisposeError, 'error.error');
    assert.sameValue(error.suppressed, secondDisposeError, 'error.suppressed');
  }
}
RunTestDisposableStackUseTwoDisposeMethodsThrow();
