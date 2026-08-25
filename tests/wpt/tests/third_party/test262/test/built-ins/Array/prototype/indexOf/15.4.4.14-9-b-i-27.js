// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf applied to Arguments object which
    implements its own property get method (number of arguments is
    greater than number of parameters)
---*/

var func = function(a, b) {
  return 0 === Array.prototype.indexOf.call(arguments, arguments[0]) &&
    3 === Array.prototype.indexOf.call(arguments, arguments[3]) &&
    -1 === Array.prototype.indexOf.call(arguments, arguments[4]);
};

assert(func(0, false, 0, true), 'func(0, false, 0, true) !== true');
