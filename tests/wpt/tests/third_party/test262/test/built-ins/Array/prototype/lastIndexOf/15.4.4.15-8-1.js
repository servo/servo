// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index(boolean)
---*/

var obj = {
  toString: function() {
    return true
  }
};
var _false = false;
var a = new Array(false, true, false, obj, _false, true, "true", undefined, 0, null, 1, "str", 0, 1);

assert.sameValue(a.lastIndexOf(true), 5, 'a[5]=true');
assert.sameValue(a.lastIndexOf(false), 4, 'a[4] =_false');
