// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index (Object)
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
var a = new Array(false, undefined, 0, false, null, {
  toString: function() {
    return "false"
  }
}, "false", obj2, obj1, obj3);

assert.sameValue(a.indexOf(obj3), 8, 'a[8] = obj1');
