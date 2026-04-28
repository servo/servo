// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - empty array to be returned if 'length' is 0
    (subclassed Array, length overridden with obj with valueOf)
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var Foo = function() {};
Foo.prototype = [1, 2, 3];
var obj = new Foo();
obj.length = {
  valueOf: function() {
    return 0;
  }
};

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult.length, 0, 'testResult.length');
