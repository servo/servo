// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws a suppressed error from errors in try and in disposal
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

// A suppressed error from an error in try block and an error in disposal
asyncTest(async function() {
  let userCodeError = new Test262Error('User code is throwing!');
  let disposeError = new Test262Error('Symbol.asyncDispose is throwing!');
  async function TestDisposeMethodAndUserCodeThrow() {
    await using x = {
      value: 1,
      [Symbol.asyncDispose]() {
        throw disposeError;
      }
    };
    throw userCodeError;
  };

  await assert.throwsAsync(
      SuppressedError, () => TestDisposeMethodAndUserCodeThrow(),
      'An error was suppressed during disposal');

  async function RunTestDisposeMethodAndUserCodeThrow() {
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
});
