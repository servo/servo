// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-43
description: >
    Object.defineProperty - 'O' is an Arguments object which
    implements its own [[GetOwnProperty]] method to access the 'name'
    property (8.12.9 step 1)
---*/

var argObj = (function() {
  return arguments;
})();

Object.defineProperty(argObj, "foo", {
  value: 12,
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperty(argObj, "foo", {
    value: 11,
    configurable: true
  });
});
assert.sameValue(argObj.foo, 12, 'argObj.foo');
