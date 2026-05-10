// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.4
description: Error thrown when accessing constructor's @@hasInstance property
info: |
    1. If Type(C) is not Object, throw a TypeError exception.
    2. Let instOfHandler be GetMethod(C,@@hasInstance).
    3. ReturnIfAbrupt(instOfHandler).
features: [Symbol.hasInstance]
---*/

var F = {};

Object.defineProperty(F, Symbol.hasInstance, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  0 instanceof F;
});
