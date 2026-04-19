// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws a suppressed error from throwing undefined in disposal.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  let firstDisposeError = undefined;
  let secondDisposeError = undefined;

  async function TestTwoDisposeMethodsThrowUndefined() {
    await using x = {
      value: 1,
      [Symbol.asyncDispose]() {
        throw firstDisposeError;
      }
    };
    await using y = {
      value: 1,
      [Symbol.asyncDispose]() {
        throw secondDisposeError;
      }
    };
  };

  await assert.throwsAsync(
      SuppressedError, () => TestTwoDisposeMethodsThrowUndefined(),
      'An error was suppressed during disposal');

  async function RunTestTwoDisposeMethodsThrowUndefined() {
    try {
      TestTwoDisposeMethodsThrowUndefined();
    } catch (error) {
      assert(
          error instanceof SuppressedError,
          'error is an instanceof SuppressedError');
      assert.sameValue(error.error, firstDisposeError, 'error.error');
      assert.sameValue(
          error.suppressed, secondDisposeError, 'error.suppressed');
    }
  }
  await RunTestTwoDisposeMethodsThrowUndefined();
});
