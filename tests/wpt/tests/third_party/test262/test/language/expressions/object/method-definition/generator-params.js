// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generator functions declared as methods honor their declared formal
    parameters.
es6id: 14.4.13
features: [generators]
---*/

var value1 = {};
var value2 = {};
var value3 = {};
var arg1, arg2, arg3;
var obj = {
  *method(a, b, c) {
    arg1 = a;
    arg2 = b;
    arg3 = c;
  }
};

obj.method(value1, value2, value3).next();

assert.sameValue(arg1, value1);
assert.sameValue(arg2, value2);
assert.sameValue(arg3, value3);
