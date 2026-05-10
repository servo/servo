// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-6
description: >
    Object.keys - inherited enumerable accessor property of 'O' is not
    defined in returned array
---*/

var proto = {};
Object.defineProperty(proto, "inheritedProp", {
  get: function() {
    return 1003;
  },
  enumerable: true,
  configurable: true
});
var Con = function() {};
Con.prototype = proto;

var obj = new Con();
Object.defineProperty(obj, "prop", {
  get: function() {
    return 1004;
  },
  enumerable: true,
  configurable: true
});

var arr = Object.keys(obj);

for (var p in arr) {
  assert.notSameValue(arr[p], "inheritedProp", 'arr[p]');
}
