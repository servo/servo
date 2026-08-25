// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - decreasing length of array with
    prototype property causes prototype index property to be visited
---*/

var arr = [0, 1, 2];

Object.defineProperty(Array.prototype, "2", {
  get: function() {
    return "prototype";
  },
  configurable: true
});

Object.defineProperty(arr, "1", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

assert.sameValue(arr.indexOf("prototype"), 2, 'arr.indexOf("prototype")');
