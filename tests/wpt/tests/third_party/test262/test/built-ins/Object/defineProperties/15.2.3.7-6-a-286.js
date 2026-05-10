// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-286
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is own
    accessor property of 'O' which is also defined in [[ParameterMap]]
    of 'O', test TypeError is thrown when updating the [[Get]]
    attribute value of 'P' which is defined as non-configurable (10.6
    [[DefineOwnProperty]] step 4)
---*/

var arg;

(function fun(a, b, c) {
  arg = arguments;
}(0, 1, 2));

function get_func1() {
  return 0;
}

Object.defineProperty(arg, "0", {
  get: get_func1,
  enumerable: false,
  configurable: false
});

function get_func2() {
  return 10;
}
assert.throws(TypeError, function() {
  Object.defineProperties(arg, {
    "0": {
      get: get_func2
    }
  });
});
var desc = Object.getOwnPropertyDescriptor(arg, "0");
assert.sameValue(desc.get, get_func1, 'desc.get');
assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, false, 'desc.configurable');
