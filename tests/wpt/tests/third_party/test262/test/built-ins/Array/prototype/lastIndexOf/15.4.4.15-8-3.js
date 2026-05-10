// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index(string)
---*/

var obj = {
  toString: function() {
    return "false"
  }
};
var szFalse = "false";
var a = new Array(szFalse, "false", "false1", undefined, 0, false, null, 1, obj, 0);

assert.sameValue(a.lastIndexOf("false"), 1, 'a.lastIndexOf("false")');
