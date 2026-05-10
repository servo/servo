// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index(undefined)
---*/

var obj = {
  toString: function() {
    return undefined;
  }
};
var _undefined1 = undefined;
var _undefined2;
var a = new Array(true, 0, false, null, 1, "undefined", obj, 1, _undefined2, _undefined1, undefined);

assert.sameValue(a.indexOf(undefined), 8, 'a[8]=_undefined2');
