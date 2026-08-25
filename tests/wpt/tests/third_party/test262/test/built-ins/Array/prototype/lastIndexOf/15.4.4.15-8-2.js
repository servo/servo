// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index(Number)
---*/

var obj = {
  toString: function() {
    return 0
  }
};
var one = 1;
var _float = -(4 / 3);
var a = new Array(+0, true, 0, -0, false, undefined, null, "0", obj, _float, -(4 / 3), -1.3333333333333, "str", one, 1, false);

assert.sameValue(a.lastIndexOf(-(4 / 3)), 10, 'a[10]=-(4/3)');
assert.sameValue(a.lastIndexOf(0), 3, 'a[3] = -0, but using === -0 and 0 are equal');
assert.sameValue(a.lastIndexOf(-0), 3, 'a[3] = -0');
assert.sameValue(a.lastIndexOf(1), 14, 'a[14] = 1');
