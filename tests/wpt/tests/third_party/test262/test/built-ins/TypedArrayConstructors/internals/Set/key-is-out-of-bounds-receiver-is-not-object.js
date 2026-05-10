// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-set
description: >
  Receiver is not an object.
info: |
  10.4.5.5 [[Set]] ( P, V, Receiver )
    ...
    i. If SameValue(O, Receiver) is true, then
      1. Perform ? TypedArraySetElement(O, numericIndex, V).
      2. Return true.
    ii. If IsValidIntegerIndex(O, numericIndex) is false, return true.

features: [TypedArray, Reflect.set]
---*/

let receiver = "not an object";

let typedArray = new Int32Array(10);

let valueOfCalled = 0;

let value = {
  valueOf() {
    valueOfCalled++;
    return 1;
  }
};

assert(Reflect.set(typedArray, 100, value, receiver), "[[Set]] succeeeds");

assert.sameValue(valueOfCalled, 0, "valueOf is not called");
