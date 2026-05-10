// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach applied to Array-like object, 'length' is
    an own data property that overrides an inherited accessor property
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 2);
}

var proto = {};

Object.defineProperty(proto, "length", {
  get: function() {
    return 3;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
Object.defineProperty(child, "length", {
  value: 2,
  configurable: true
});
child[0] = 12;
child[1] = 11;
child[2] = 9;

Array.prototype.forEach.call(child, callbackfn);

assert(result, 'result !== true');
