// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf applied to Arguments object which
    implements its own property get method
---*/

var targetObj = function() {};
var func = function(a, b) {
  arguments[2] = function() {};
  return Array.prototype.lastIndexOf.call(arguments, targetObj) === 1 &&
    Array.prototype.lastIndexOf.call(arguments, arguments[2]) === -1;
};

assert(func(0, targetObj), 'func(0, targetObj) !== true');
