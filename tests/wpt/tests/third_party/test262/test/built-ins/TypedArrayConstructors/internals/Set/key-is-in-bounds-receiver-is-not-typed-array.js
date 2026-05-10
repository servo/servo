// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-set
description: >
  Receiver is not the typed array object.
info: |
  10.4.5.5 [[Set]] ( P, V, Receiver )
    ...
    i. If SameValue(O, Receiver) is true, then
      ...
    ii. If IsValidIntegerIndex(O, numericIndex) is false, return true.
  2. Return ? OrdinarySet(O, P, V, Receiver).

features: [TypedArray, Reflect.set]
---*/

let receiver = {};

let typedArray = new Int32Array(10);

let valueOfCalled = 0;

let value = {
  valueOf() {
    valueOfCalled++;
    return 1;
  }
};

assert(Reflect.set(typedArray, 0, value, receiver), "[[Set]] succeeeds");

assert.sameValue(valueOfCalled, 0, "valueOf is not called");

assert.sameValue(receiver[0], value, "value assigned to receiver[0]");
