// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is own data
    property that overrides an inherited data property on an Array
---*/

var kValue = "abc";
var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    testResult = (val === kValue);
  }
}

var proto = {
  5: 100
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[5] = kValue;
child.length = 10;

Array.prototype.forEach.call(child, callbackfn);

assert(testResult, 'testResult !== true');
