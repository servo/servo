// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Rejects the promise if setting an element fails on an instance of a custom
  this-value
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

  await assert.throwsAsync(TypeError, () => Array.fromAsync.call(MyArray, [0, 1, 2]), "Promise rejected if defining element fails");
});
