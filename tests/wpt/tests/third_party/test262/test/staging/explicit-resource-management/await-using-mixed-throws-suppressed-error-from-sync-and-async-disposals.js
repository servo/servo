// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws a suppressed error from errors in  sync and async disposal.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let firstDisposeError = new Test262Error('The Symbol.dispose is throwing!');
  let secondDisposeError =
      new Test262Error('The Symbol.asyncDispose is throwing!');

  async function TestTwoDisposeMethodsThrow() {
    using x = {
      value: 1,
      [Symbol.dispose]() {
        throw firstDisposeError;
      }
    };
    await using y = {
      value: 1,
      async[Symbol.asyncDispose]() {
        throw secondDisposeError;
      }
    };
  };

  await assert.throwsAsync(
      SuppressedError, () => TestTwoDisposeMethodsThrow(),
      'An error was suppressed during disposal');

  async function RunTestTwoDisposeMethodsThrow() {
    try {
      TestTwoDisposeMethodsThrow();
    } catch (error) {
      assert(
          error instanceof SuppressedError,
          'error is an instanceof SuppressedError');
      assert.sameValue(error.error, firstDisposeError, 'error.error');
      assert.sameValue(
          error.suppressed, secondDisposeError, 'error.suppressed');
    }
  }
  await RunTestTwoDisposeMethodsThrow();
});
