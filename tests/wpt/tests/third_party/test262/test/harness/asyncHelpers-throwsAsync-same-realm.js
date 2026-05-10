// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Thenables that reject with instances of the realm specified constructor function
    do not satisfy the assertion with cross realms collisions.
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  var intrinsicTypeError = TypeError;
  var caught = false;
  var realmGlobal = $262.createRealm().global;

  const p = assert.throwsAsync(TypeError, async function () {
    throw new realmGlobal.TypeError();
  });
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
      "assert.throwsAsync did not reject when a different realm's error was thrown"
    );
  }
});
