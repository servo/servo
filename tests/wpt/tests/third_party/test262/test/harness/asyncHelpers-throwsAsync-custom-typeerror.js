// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Thenables that reject with instances of the specified constructor function
    satisfy the assertion, without collision with error constructors of the same name.
flags: [async]
includes: [asyncHelpers.js]
---*/

var intrinsicTypeError = TypeError;

asyncTest(async function () {
  function TypeError() {}
  var caught = false;

  var p = assert.throwsAsync(
    TypeError,
    async function () {
      throw new TypeError();
    },
    "Throws an instance of the matching custom TypeError"
  );
  assert(p instanceof Promise);
  await p;

  p = assert.throwsAsync(intrinsicTypeError, async function () {
    throw new TypeError();
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
      "assert.throwsAsync did not reject a collision of constructor names"
    );
  }

  caught = false;

  p = assert.throwsAsync(TypeError, async function () {
    throw new intrinsicTypeError();
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
      "assert.throwsAsync did not reject a collision of constructor names"
    );
  }
})
