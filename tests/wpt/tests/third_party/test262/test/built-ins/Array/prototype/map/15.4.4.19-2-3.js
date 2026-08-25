// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - applied to Array-like object, 'length' is an
    own data property that overrides an inherited data property
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}

var proto = {
  length: 3
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 2;
child[0] = 12;
child[1] = 11;
child[2] = 9;

var testResult = Array.prototype.map.call(child, callbackfn);

assert.sameValue(testResult.length, 2, 'testResult.length');
