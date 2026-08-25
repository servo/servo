// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own data
    property that overrides an inherited data property on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    return val === 100;
  } else {
    return true;
  }
}

var proto = {
  0: 11,
  5: 100
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[5] = "abc";
child.length = 10;

assert.sameValue(Array.prototype.every.call(child, callbackfn), false, 'Array.prototype.every.call(child, callbackfn)');
