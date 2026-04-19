// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to Array-like object, 'length'
    is an inherited accessor property
---*/

var accessed = false;
var Con = function() {};


function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return obj.length === 2;
}

var proto = {};

Object.defineProperty(proto, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

Con.prototype = proto;

var child = new Con();
child[0] = 12;
child[1] = 11;
child[2] = 9;


assert(Array.prototype.reduceRight.call(child, callbackfn, 11), 'Array.prototype.reduceRight.call(child, callbackfn, 11) !== true');
assert(accessed, 'accessed !== true');
