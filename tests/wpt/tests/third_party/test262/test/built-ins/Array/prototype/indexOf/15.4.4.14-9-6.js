// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index(null)
---*/

var obj = {
  toString: function() {
    return null
  }
};
var _null = null;
var a = new Array(true, undefined, 0, false, _null, 1, "str", 0, 1, obj, true, false, null);

assert.sameValue(a.indexOf(null), 4, 'a[4]=_null');
