// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    assert.throwsAsync returns a promise that rejects when invoked with a single argument
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  var caught = false;
  const p = assert.throwsAsync(function () {});
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
      "assert.throwsAsync did not reject when invoked with a single argumemnt"
    );
  }
});
