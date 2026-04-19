// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - value of 'fromIndex' is an Object, which
    has an own toString method
---*/

// objects inherit the default valueOf() method from Object
// that simply returns itself. Since the default valueOf() method
// does not return a primitive value, ES next tries to convert the object
// to a number by calling its toString() method and converting the
// resulting string to a number.
var fromIndex = {
  toString: function() {
    return '1';
  }
};

assert.sameValue([0, true].indexOf(true, fromIndex), 1, '[0, true].indexOf(true, fromIndex)');
