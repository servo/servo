// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to the Array-like object that
    'length' is inherited accessor property without a get function
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return curVal > 10;
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

assert.sameValue(Array.prototype.reduceRight.call(child, callbackfn, 111), 111, 'Array.prototype.reduceRight.call(child, callbackfn, 111)');
assert.sameValue(accessed, false, 'accessed');
