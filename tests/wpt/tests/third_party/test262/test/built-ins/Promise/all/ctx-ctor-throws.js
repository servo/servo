// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.all` invoked on a constructor value that throws an error
es6id: 25.4.4.1
info: |
    1. Let C be the this value.
    [...]
    6. Let promiseCapability be NewPromiseCapability(C).
    7. ReturnIfAbrupt(promiseCapability).

    25.4.1.5 NewPromiseCapability
    [...]
    6. Let promise be Construct(C, «executor»).
    7. ReturnIfAbrupt(promise).
---*/

var CustomPromise = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  Promise.all.call(CustomPromise);
});
