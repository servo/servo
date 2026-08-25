// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.4
description: >
    Error thrown when constructor's @@hasInstance property is defined but not callable
info: |
    1. If Type(C) is not Object, throw a TypeError exception.
    2. Let instOfHandler be GetMethod(C,@@hasInstance).
    3. ReturnIfAbrupt(instOfHandler).
    4. If instOfHandler is not undefined, then
       a. Return ToBoolean(Call(instOfHandler, C, «O»)).
    5. If IsCallable(C) is false, throw a TypeError exception.
features: [Symbol.hasInstance]
---*/

var F = {};

F[Symbol.hasInstance] = null;

assert.throws(TypeError, function() {
  0 instanceof F;
});
