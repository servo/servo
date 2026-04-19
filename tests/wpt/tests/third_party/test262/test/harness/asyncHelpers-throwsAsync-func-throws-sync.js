// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    assert.throwsAsync returns a promise that rejects if func or the inner thenable synchronously throws.
flags: [async]
includes: [asyncHelpers.js]
---*/

async function checkRejects(func) {
  var caught = false;
  const p = assert.throwsAsync(Test262Error, func);
  assert(p instanceof Promise, "assert.throwsAsync should return a promise");
  try {
    await p;
  } catch (e) {
    caught = true;
    assert.sameValue(
      e.constructor,
      Test262Error,
      "throwsAsync should reject improper function with a Test262Error"
    );
  } finally {
    assert(
      caught,
      "assert.throwsAsync did not reject improper function " + func
    );
  }
}

asyncTest(async function () {
  await checkRejects(function () {
    throw new Error();
  });
  await checkRejects(function () {
    throw new Test262Error();
  });
  await checkRejects(function () {
    return {
      then: function () {
        throw new Error();
      },
    };
  });
  await checkRejects(function () {
    return {
      then: function () {
        throw new Test262Error();
      },
    };
  });
});
