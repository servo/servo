// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-15
description: >
    Object.create - argument 'Properties' is the Aguments object
    (15.2.3.7 step 2)
---*/

var result = false;

var argObj = (function() {
  return arguments;
})();

Object.defineProperty(argObj, "prop", {
  get: function() {
    result = ('[object Arguments]' === Object.prototype.toString.call(this));
    return {};
  },
  enumerable: true
});

var newObj = Object.create({}, argObj);

assert(result, 'result !== true');
assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
