// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.reject` invoked on a non-constructor value
es6id: 25.4.4.4
info: |
    [...]
    3. Let promiseCapability be NewPromiseCapability(C).
    4. ReturnIfAbrupt(promiseCapability).
---*/

assert.throws(TypeError, function() {
  Promise.reject.call(eval);
});
