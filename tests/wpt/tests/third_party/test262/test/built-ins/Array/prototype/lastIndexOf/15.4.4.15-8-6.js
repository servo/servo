// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index(null)
---*/

var obj = {
  toString: function() {
    return null
  }
};
var _null = null;
var a = new Array(true, undefined, 0, false, null, 1, "str", 0, 1, null, true, false, undefined, _null, "null", undefined, "str", obj);

assert.sameValue(a.lastIndexOf(null), 13, 'a.lastIndexOf(null)');
