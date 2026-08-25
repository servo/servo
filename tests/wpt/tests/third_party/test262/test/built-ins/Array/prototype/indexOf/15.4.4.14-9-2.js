// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index (Number)
---*/

var obj = {
  toString: function() {
    return 0
  }
};
var one = 1;
var _float = -(4 / 3);
var a = new Array(false, undefined, null, "0", obj, -1.3333333333333, "str", -0, true, +0, one, 1, 0, false, _float, -(4 / 3));

assert.sameValue(a.indexOf(-(4 / 3)), 14, 'a[14]=_float===-(4/3)');
assert.sameValue(a.indexOf(0), 7, 'a[7] = +0, 0===+0');
assert.sameValue(a.indexOf(-0), 7, 'a[7] = +0, -0===+0');
assert.sameValue(a.indexOf(1), 10, 'a[10] =one=== 1');
