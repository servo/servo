// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Rejects the promise if setting the length fails on an instance of a custom
  this-value
info: |
  3.j.ii.4.a. Perform ? Set(_A_, *"length"*, 𝔽(_k_), *true*).
  ...
  3.k.viii. Perform ? Set(_A_, *"length"*, 𝔽(_len_), *true*)
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  class MyArray {
    set length(v) {
      throw new Test262Error("setter of length property throws")
    }
  }

  await assert.throwsAsync(Test262Error, () => Array.fromAsync.call(MyArray, [0, 1, 2]), "Promise rejected if setting length fails");

  await assert.throwsAsync(Test262Error, () => Array.fromAsync.call(MyArray, {
    length: 3,
    0: 0,
    1: 1,
    2: 2
  }), "Promise rejected if setting length from array-like fails");
});
