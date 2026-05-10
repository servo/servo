// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach doesn't call callbackfn if 'length' is 0
    (subclassed Array, length overridden with obj w/o valueOf
    (toString))
---*/

foo.prototype = new Array(1, 2, 3);

function foo() {}
var f = new foo();

var o = {
  toString: function() {
    return '0';
  }
};
f.length = o;

// objects inherit the default valueOf method of the Object object;
// that simply returns the itself. Since the default valueOf() method
// does not return a primitive value, ES next tries to convert the object
// to a number by calling its toString() method and converting the
// resulting string to a number.
var callCnt = 0;

function cb() {
  callCnt++
}
var i = f.forEach(cb);

assert.sameValue(callCnt, 0, 'callCnt');
