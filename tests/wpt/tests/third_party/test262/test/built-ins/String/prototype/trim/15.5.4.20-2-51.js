// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-51
description: >
    String.prototype.trim - 'this' is a Arguments Object that converts
    to a string
---*/

var argObj = function() {
  return arguments;
}(1, 2, true);

assert.sameValue(String.prototype.trim.call(argObj), "[object Arguments]", 'String.prototype.trim.call(argObj)');
