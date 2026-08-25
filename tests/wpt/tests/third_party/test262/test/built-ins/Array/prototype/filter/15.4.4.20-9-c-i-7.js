// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element to be retrieved is inherited data
    property on an Array-like object
---*/

var kValue = 'abc';

function callbackfn(val, idx, obj) {
  return (idx === 5) && (val === kValue);
}

var proto = {
  5: kValue
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

var newArr = Array.prototype.filter.call(child, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], kValue, 'newArr[0]');
