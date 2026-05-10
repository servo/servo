// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is inherited data
    property on an Array-like object
---*/

var kValue = 'abc';

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    return val !== kValue;
  } else {
    return true;
  }
}

var proto = {
  5: kValue
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

assert.sameValue(Array.prototype.every.call(child, callbackfn), false, 'Array.prototype.every.call(child, callbackfn)');
