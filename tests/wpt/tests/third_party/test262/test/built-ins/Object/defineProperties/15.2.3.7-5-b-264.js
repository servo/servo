// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-264
description: >
    Object.defineProperties - TypeError is thrown if both 'get'
    property and 'writable' property of 'descObj' are present (8.10.5
    step 9.a)
---*/

var getFun = function() {};

var obj = {};
assert.throws(TypeError, function() {
  Object.defineProperties(obj, {
    "prop": {
      writable: true,
      get: getFun
    }
  });
});
