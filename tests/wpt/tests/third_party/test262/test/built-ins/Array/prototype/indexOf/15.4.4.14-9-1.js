// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index (boolean)
---*/

var obj = {
  toString: function() {
    return true
  }
};
var _false = false;
var a = [obj, "true", undefined, 0, _false, null, 1, "str", 0, 1, true, false, true, false];

assert.sameValue(a.indexOf(true), 10, 'a[10]=true');
assert.sameValue(a.indexOf(false), 4, 'a[4] =_false');
