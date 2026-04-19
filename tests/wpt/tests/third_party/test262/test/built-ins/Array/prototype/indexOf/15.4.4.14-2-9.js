// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - 'length' is own accessor property that
    overrides an inherited accessor property
---*/

var proto = {};
Object.defineProperty(proto, "length", {
  get: function() {
    return 0;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[1] = true;

Object.defineProperty(child, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call(child, true), 1, 'Array.prototype.indexOf.call(child, true)');
