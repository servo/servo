// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf applied to Arguments object which
    implements its own property get method
---*/

var func = function(a, b) {
  arguments[2] = false;
  return Array.prototype.indexOf.call(arguments, true) === 1 &&
    Array.prototype.indexOf.call(arguments, false) === -1;
};

assert(func(0, true), 'func(0, true) !== true');
