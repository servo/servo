// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-280
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is own
    accessor property of 'O' which is also defined in [[ParameterMap]]
    of 'O', and 'desc' is accessor descriptor, test updating multiple
    attribute values of 'P' (10.6 [[DefineOwnProperty]] step 3)
---*/

var arg;

(function fun(a, b, c) {
  arg = arguments;
}(0, 1, 2));

function get_func1() {
  return 10;
}

Object.defineProperty(arg, "0", {
  get: get_func1,
  enumerable: true,
  configurable: true
});

function get_func2() {
  return 20;
}

Object.defineProperties(arg, {
  "0": {
    get: get_func2,
    enumerable: false,
    configurable: false
  }
});

var desc = Object.getOwnPropertyDescriptor(arg, "0");

assert.sameValue(desc.get, get_func2, 'desc.get');
assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
assert.sameValue(desc.configurable, false, 'desc.configurable');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
