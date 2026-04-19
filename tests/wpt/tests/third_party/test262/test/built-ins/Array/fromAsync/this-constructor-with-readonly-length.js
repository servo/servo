// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Promise is rejected if length property on an instance of a custom this-value
  is non-writable
info: |
  3.j.ii.4.a. Perform ? Set(_A_, *"length"*, 𝔽(_k_), *true*).
  ...
  3.k.viii. Perform ? Set(_A_, *"length"*, 𝔽(_len_), *true*).

  Note that there is no difference between strict mode and sloppy mode, because
  we are not following runtime evaluation semantics.
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  function MyArray() {
    Object.defineProperty(this, "length", {
      enumerable: true,
      writable: false,
      configurable: true,
      value: 99
    });
  }

  await assert.throwsAsync(TypeError, () => Array.fromAsync.call(MyArray, [0, 1, 2]), "Setting read-only length fails");
  await assert.throwsAsync(TypeError, () => Array.fromAsync.call(MyArray, {
    length: 3,
    0: 0,
    1: 1,
    2: 2
  }), "Setting read-only length fails in array-like case");
});
