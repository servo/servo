// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - 'length' is an own data property that
    overrides an inherited data property on an Array-like object
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 2);
}

var proto = {
  length: 3
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 2;
child[0] = 12;
child[1] = 11;
child[2] = 9;

Array.prototype.forEach.call(child, callbackfn);

assert(result, 'result !== true');
