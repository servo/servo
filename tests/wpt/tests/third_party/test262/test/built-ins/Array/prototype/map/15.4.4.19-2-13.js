// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - applied to the Array-like object when
    'length' is inherited accessor property without a get function
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}

var proto = {};
Object.defineProperty(proto, "length", {
  set: function() {},
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[0] = 11;
child[1] = 12;

var testResult = Array.prototype.map.call(child, callbackfn);

assert.sameValue(testResult.length, 0, 'testResult.length');
