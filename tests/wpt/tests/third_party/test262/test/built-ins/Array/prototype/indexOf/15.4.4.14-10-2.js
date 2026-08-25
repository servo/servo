// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf returns -1 if 'length' is 0 and does not
    access any other properties
---*/

var accessed = false;
var f = {
  length: 0
};
Object.defineProperty(f, "0", {
  get: function() {
    accessed = true;
    return 1;
  }
});


var i = Array.prototype.indexOf.call(f, 1);


assert.sameValue(i, -1, 'i');
assert.sameValue(accessed, false, 'accessed');
