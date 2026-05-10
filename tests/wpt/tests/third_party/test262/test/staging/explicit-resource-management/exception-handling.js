// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test exception handling.
features: [explicit-resource-management]
---*/

// User code throws -----------------------------
function TestUserCodeThrowsBeforeUsingStatements() {
  {
    throw new Test262Error('User code is throwing!');
    using x = {
      value: 1,
      [Symbol.dispose]() {
        return 42;
      }
    };
  }
};

assert.throws(
    Test262Error, () => TestUserCodeThrowsBeforeUsingStatements(),
    'User code is throwing!');

function TestUserCodeThrowsAfterUsingStatements() {
  {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        return 42;
      }
    };
    throw new Test262Error('User code is throwing!');
  }
};
assert.throws(
    Test262Error, () => TestUserCodeThrowsAfterUsingStatements(),
    'User code is throwing!');

// Dispose method throws -----------------------------
function TestDisposeMethodThrows() {
  {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        throw new Test262Error('Symbol.dispose is throwing!');
      }
    };
  }
};
assert.throws(
    Test262Error, () => TestDisposeMethodThrows(),
    'Symbol.dispose is throwing!');

// Dispose method is not a function -----------------------------
function TestDisposeMethodNotAFunction() {
  using x = 42;
};
assert.throws(
    TypeError, () => TestDisposeMethodNotAFunction(),
    'Symbol.Dispose is not a function');

// A suppressed error from an error in try block and an error in disposal
// -----------------------------
let userCodeError = new Test262Error('User code is throwing!');
let disposeError = new Test262Error('Symbol.dispose is throwing!');

function TestDisposeMethodAndUserCodeThrow() {
  {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        throw disposeError;
      }
    };
    throw userCodeError;
  }
};

assert.throws(
    SuppressedError, () => TestDisposeMethodAndUserCodeThrow(),
    'An error was suppressed during disposal');

function RunTestDisposeMethodAndUserCodeThrow() {
  try {
    TestDisposeMethodAndUserCodeThrow();
  } catch (error) {
    assert(
        error instanceof SuppressedError,
        'error is an instanceof SuppressedError');
    assert.sameValue(error.error, disposeError, 'error.error');
    assert.sameValue(error.suppressed, userCodeError, 'error.suppressed');
  }
}
RunTestDisposeMethodAndUserCodeThrow();

// A suppressed error from two errors in disposal -----------------------------
let firstDisposeError =
    new Test262Error('The first Symbol.dispose is throwing!');
let secondDisposeError =
    new Test262Error('The second Symbol.dispose is throwing!');

function TestTwoDisposeMethodsThrow() {
  {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        throw firstDisposeError;
      }
    };
    using y = {
      value: 1,
      [Symbol.dispose]() {
        throw secondDisposeError;
      }
    };
  }
};

assert.throws(
    SuppressedError, () => TestTwoDisposeMethodsThrow(),
    'An error was suppressed during disposal');

function RunTestTwoDisposeMethodsThrow() {
  try {
    TestTwoDisposeMethodsThrow();
  } catch (error) {
    assert(
        error instanceof SuppressedError,
        'error is an instanceof SuppressedError');
    assert.sameValue(error.error, firstDisposeError, 'error.error');
    assert.sameValue(error.suppressed, secondDisposeError, 'error.suppressed');
  }
}
RunTestTwoDisposeMethodsThrow();
