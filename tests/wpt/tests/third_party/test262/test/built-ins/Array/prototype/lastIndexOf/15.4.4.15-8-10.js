// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    note that prior to the finally ES5 draft SameValue was used for comparisions
    and hence NaNs could be found using lastIndexOf *
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index (NaN)
---*/

var _NaN = NaN;
var a = new Array("NaN", _NaN, NaN, undefined, 0, false, null, {
  toString: function() {
    return NaN
  }
}, "false");

assert.sameValue(a.lastIndexOf(NaN), -1, 'NaN matches nothing, not even itself');
