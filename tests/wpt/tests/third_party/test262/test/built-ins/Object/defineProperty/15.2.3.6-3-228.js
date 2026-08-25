// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-228
description: >
    Object.defineProperty - 'Attributes' is an Arguments object which
    implements its own [[Get]] method to access the 'get' property
    (8.10.5 step 7.a)
---*/

var obj = {};

var argObj = (function() {
  return arguments;
})();

argObj.get = function() {
  return "argumentGetProperty";
};

Object.defineProperty(obj, "property", argObj);

assert.sameValue(obj.property, "argumentGetProperty", 'obj.property');
