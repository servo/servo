// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is inherited data
    property on an Array-like object
---*/

var kValue = 'abc';

function callbackfn(val, idx, obj) {
  if (5 === idx) {
    return kValue === val;
  }
  return false;
}

var proto = {
  5: kValue
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

assert(Array.prototype.some.call(child, callbackfn), 'Array.prototype.some.call(child, callbackfn) !== true');
