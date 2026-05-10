// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is inherited
    accessor property without a get function on an Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return typeof val === "undefined";
  }
  return false;
}

var proto = {};
Object.defineProperty(proto, "1", {
  set: function() {},
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 2;

assert(Array.prototype.some.call(child, callbackfn), 'Array.prototype.some.call(child, callbackfn) !== true');
