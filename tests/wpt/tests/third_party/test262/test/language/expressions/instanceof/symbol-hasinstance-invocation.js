// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.4
description: >
    Arguments and 'this' value when invoking constructor's @@hasInstance property
info: |
    1. If Type(C) is not Object, throw a TypeError exception.
    2. Let instOfHandler be GetMethod(C,@@hasInstance).
    3. ReturnIfAbrupt(instOfHandler).
    4. If instOfHandler is not undefined, then
       a. Return ToBoolean(Call(instOfHandler, C, «O»)).
features: [Symbol.hasInstance]
---*/

var F = {};
var callCount = 0;
var thisValue, args;

F[Symbol.hasInstance] = function() {
  thisValue = this;
  args = arguments;
  callCount += 1;
};

0 instanceof F;

assert.sameValue(callCount, 1);
assert.sameValue(thisValue, F);
assert.sameValue(args.length, 1);
assert.sameValue(args[0], 0);
