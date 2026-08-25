// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Promise.any() does not access a `Symbol.species` property of the `this` value
info: |
  Let C be the this value.
  Let promiseCapability be ? NewPromiseCapability(C).
  ...

features: [Promise.any, Symbol.species]
---*/
function C(executor) {
  executor(() => {}, () => {});
}

Object.defineProperty(C, Symbol.species, {
  get() {
    throw new Test262Error("Getter for Symbol.species called");
  }
});

C.resolve = function() {
  throw new Test262Error("C.resolve called unexpectedly");
};

Promise.any.call(C, [1]);
