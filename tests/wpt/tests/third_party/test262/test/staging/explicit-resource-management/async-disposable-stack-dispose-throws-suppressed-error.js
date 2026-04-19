// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: disposeAsync() throws a suppressed error.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let firstDisposeError =
      new Test262Error('The first Symbol.asyncDispose is throwing!');
  let secondDisposeError =
      new Test262Error('The second Symbol.asyncDispose is throwing!');

  async function TestAsyncDisposableStackUseTwoDisposeMethodsThrow() {
    {
      let stack = new AsyncDisposableStack();
      const firstDisposable = {
        value: 1,
        [Symbol.asyncDispose]() {
          throw firstDisposeError;
        }
      };
      const secondDisposable = {
        value: 1,
        [Symbol.asyncDispose]() {
          throw secondDisposeError;
        }
      };
      stack.use(firstDisposable);
      stack.use(secondDisposable);
      await stack.disposeAsync();
    }
  };

  await assert.throwsAsync(
      SuppressedError,
      () => TestAsyncDisposableStackUseTwoDisposeMethodsThrow(),
      'An error was suppressed during disposal');

  async function RunTestAsyncDisposableStackUseTwoDisposeMethodsThrow() {
    try {
      await TestAsyncDisposableStackUseTwoDisposeMethodsThrow();
    } catch (error) {
      assert(
          error instanceof SuppressedError,
          'error is an instanceof SuppressedError');
      assert.sameValue(error.error, firstDisposeError, 'error.error');
      assert.sameValue(
          error.suppressed, secondDisposeError, 'error.suppressed');
    }
  }
  await RunTestAsyncDisposableStackUseTwoDisposeMethodsThrow();
});
