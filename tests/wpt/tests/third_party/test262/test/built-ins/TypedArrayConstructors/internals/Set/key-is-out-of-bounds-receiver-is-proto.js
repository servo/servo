// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-set
description: >
  Receiver is an object in the prototype chain.
info: |
  10.4.5.5 [[Set]] ( P, V, Receiver )
    ...
    i. If SameValue(O, Receiver) is true, then
      1. Perform ? TypedArraySetElement(O, numericIndex, V).
      2. Return true.
    ii. If IsValidIntegerIndex(O, numericIndex) is false, return true.

  10.4.5.16 TypedArraySetElement ( O, index, value )
    1. If O.[[ContentType]] is bigint, let numValue be ? ToBigInt(value).
    2. Otherwise, let numValue be ? ToNumber(value).
    ...

features: [TypedArray, Reflect.set]
---*/

let receiver = new Int32Array(10);

// |receiver| is in the prototype chain of |obj|.
let obj = Object.create(receiver);

let valueOfCalled = 0;

let value = {
  valueOf() {
    valueOfCalled++;
    return 1;
  }
};

assert(Reflect.set(obj, 100, value, receiver), "[[Set]] succeeeds");

assert.sameValue(valueOfCalled, 1, "valueOf is called exactly once");
