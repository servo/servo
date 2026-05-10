// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf - 'length' is inherited accessor property
---*/

var proto = {};
Object.defineProperty(proto, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var childOne = new Con();
childOne[1] = true;
var childTwo = new Con();
childTwo[2] = true;

assert.sameValue(Array.prototype.indexOf.call(childOne, true), 1, 'Array.prototype.indexOf.call(childOne, true)');
assert.sameValue(Array.prototype.indexOf.call(childTwo, true), -1, 'Array.prototype.indexOf.call(childTwo, true)');
