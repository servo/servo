// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Thenables that reject with instances of the specified native Error constructor
    satisfy the assertion.
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  var p = assert.throwsAsync(Error, async function () {
    throw new Error();
  });
  assert(p instanceof Promise);
  await p;
  p = assert.throwsAsync(EvalError, async function () {
    throw new EvalError();
  });
  assert(p instanceof Promise);
  await p;
  p = assert.throwsAsync(RangeError, async function () {
    throw new RangeError();
  });
  assert(p instanceof Promise);
  await p;
  p = assert.throwsAsync(ReferenceError, async function () {
    throw new ReferenceError();
  });
  assert(p instanceof Promise);
  await p;
  p = assert.throwsAsync(SyntaxError, async function () {
    throw new SyntaxError();
  });
  assert(p instanceof Promise);
  await p;
  p = assert.throwsAsync(TypeError, async function () {
    throw new TypeError();
  });
  assert(p instanceof Promise);
  await p;
  p = assert.throwsAsync(URIError, async function () {
    throw new URIError();
  });
  assert(p instanceof Promise);
  await p;
});
