// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The assertion fails when invoked without arguments.
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  var caught = false;

  const p = assert.throwsAsync();
  assert(p instanceof Promise);
  try {
    await p;
  } catch (err) {
    caught = true;
    assert.sameValue(
      err.constructor,
      Test262Error,
      "Expected a Test262Error, but a '" +
        err.constructor.name +
        "' was thrown."
    );
  } finally {
    assert(
      caught,
      "assert.throwsAsync did not reject when invoked without arguments"
    );
  }
});
