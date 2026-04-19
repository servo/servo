// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Closes an async iterator if setting an element fails on an instance of a
  custom this-value
info: |
  3.j.ii.8. Let _defineStatus_ be CreateDataPropertyOrThrow(_A_, _Pk_, _mappedValue_).
  9. If _defineStatus_ is an abrupt completion, return ? AsyncIteratorClose(_iteratorRecord_, _defineStatus_).
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  function MyArray() {
    Object.defineProperty(this, 0, {
      enumerable: true,
      writable: true,
      configurable: false,
      value: 0
    });
  }

  let closed = false;
  const iterator = {
    next() {
      return Promise.resolve({ value: 1, done: false });
    },
    return() {
      closed = true;
      return Promise.resolve({ done: true });
    },
    [Symbol.asyncIterator]() {
      return this;
    }
  }

  await assert.throwsAsync(TypeError, () => Array.fromAsync.call(MyArray, iterator), "Promise rejected if defining element fails");
  assert(closed, "element define failure should close iterator");
});
