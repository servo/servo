// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Overwrites non-writable element properties on an instance of a custom
  this-value
info: |
  3.j.ii.8. Let _defineStatus_ be CreateDataPropertyOrThrow(_A_, _Pk_, _mappedValue_).
  ...
  3.k.vii.6. Perform ? CreateDataPropertyOrThrow(_A_, _Pk_, _mappedValue_).
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  function MyArray() {
    this.length = 4;
    for (let ix = 0; ix < this.length; ix++) {
      Object.defineProperty(this, ix, {
        enumerable: true,
        writable: false,
        configurable: true,
        value: 99
      });
    }
  }

  let result = await Array.fromAsync.call(MyArray, [0, 1, 2]);
  assert.sameValue(result.length, 3, "Length property is overwritten");
  assert.sameValue(result[0], 0, "Read-only element 0 is overwritten");
  assert.sameValue(result[1], 1, "Read-only element 1 is overwritten");
  assert.sameValue(result[2], 2, "Read-only element 2 is overwritten");
  assert.sameValue(result[3], 99, "Element 3 is not overwritten");

  result = await Array.fromAsync.call(MyArray, {
    length: 3,
    0: 0,
    1: 1,
    2: 2,
    3: 3  // ignored
  });
  assert.sameValue(result.length, 3, "Length property is overwritten");
  assert.sameValue(result[0], 0, "Read-only element 0 is overwritten");
  assert.sameValue(result[1], 1, "Read-only element 1 is overwritten");
  assert.sameValue(result[2], 2, "Read-only element 2 is overwritten");
  assert.sameValue(result[3], 99, "Element 3 is not overwritten");
});
