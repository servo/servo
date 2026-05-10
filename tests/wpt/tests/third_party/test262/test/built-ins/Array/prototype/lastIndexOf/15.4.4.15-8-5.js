// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index(Object)
---*/

var obj1 = {
  toString: function() {
    return "false"
  }
};
var obj2 = {
  toString: function() {
    return "false"
  }
};
var obj3 = obj1;
var a = new Array(obj2, obj1, obj3, false, undefined, 0, false, null, {
  toString: function() {
    return "false"
  }
}, "false");

assert.sameValue(a.lastIndexOf(obj3), 2, 'a.lastIndexOf(obj3)');
