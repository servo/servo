// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter applied to Array-like object, 'length' is
    an inherited data property
---*/

function callbackfn(val, idx, obj) {
  return obj.length === 2;
}

var proto = {
  length: 2
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[0] = 12;
child[1] = 11;
child[2] = 9;

var newArr = Array.prototype.filter.call(child, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
