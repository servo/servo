// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test exception handling when user code throws.
includes: [asyncHelpers.js]
flags: [async]
features: [explicit-resource-management]
---*/

// User code throws -----------------------------
asyncTest(async function() {
  async function TestUserCodeThrowsAfterUsingStatements() {
    await using x = {
      value: 1,
      [Symbol.asyncDispose]() {
        return 42;
      }
    };
    throw new Test262Error('User code is throwing!');
  };
  await assert.throwsAsync(
      Test262Error, () => TestUserCodeThrowsAfterUsingStatements(),
      'User code is throwing!');
});
