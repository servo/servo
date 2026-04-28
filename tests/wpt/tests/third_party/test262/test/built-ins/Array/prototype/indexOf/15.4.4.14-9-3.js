// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index(string)
---*/

var obj = {
  toString: function() {
    return "false"
  }
};
var szFalse = "false";
var a = new Array("false1", undefined, 0, false, null, 1, obj, 0, szFalse, "false");

assert.sameValue(a.indexOf("false"), 8, 'a[8]=szFalse');
