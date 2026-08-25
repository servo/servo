// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-46
description: >
    String.prototype.trim - 'this' is a Function Object that converts
    to a string
---*/

var funObj = function() {
  return arguments;
};

assert.sameValue(typeof(String.prototype.trim.call(funObj)), "string", 'typeof(String.prototype.trim.call(funObj))');
