// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
for (let ctor of typedArrayConstructors) {
  let arr = new ctor([1, 2, 3, 4, 5, 6, 7, 8]);

  arr.buffer.constructor = {
    get [Symbol.species]() {
      throw new Error("unexpected @@species access");
    }
  };

  for (let ctor2 of typedArrayConstructors) {
    let arr2 = new ctor2(arr);

    assert.sameValue(Object.getPrototypeOf(arr2.buffer), ArrayBuffer.prototype);
    assert.sameValue(arr2.buffer.constructor, ArrayBuffer);
  }
}

