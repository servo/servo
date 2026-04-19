// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.allSettled() does not retrieve `Symbol.species` property of the `this` value
esid: sec-promise.allsettled
info: |
  1. Let C be the this value.
  2. If Type(C) is not Object, throw a TypeError exception.
  3. Let promiseCapability be ? NewPromiseCapability(C).
  ...
features: [Promise.allSettled, Symbol.species]
---*/

function C(executor) {
  executor(function() {}, function() {});
}
Object.defineProperty(C, Symbol.species, {
  get() {
    throw new Test262Error('Getter for Symbol.species called');
  }
});

C.resolve = function() {
  throw new Test262Error();
};

Promise.allSettled.call(C, []);
