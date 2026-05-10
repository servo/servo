// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - element to be retrieved is own data property
    that overrides an inherited data property on an Array-like object
---*/

var kValue = "abc";

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    return val === kValue;
  }
  return false;
}

var proto = {
  5: 12,
  length: 10
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[5] = kValue;

var testResult = Array.prototype.map.call(child, callbackfn);

assert.sameValue(testResult[5], true, 'testResult[5]');
