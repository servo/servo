// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-39
description: >
    String.prototype.trim - 'this' is an object which has an own
    valueOf method
---*/

var obj = {
  valueOf: function() {
    return "abc";
  }
};

assert.sameValue(String.prototype.trim.call(obj), "[object Object]", 'String.prototype.trim.call(obj)');
