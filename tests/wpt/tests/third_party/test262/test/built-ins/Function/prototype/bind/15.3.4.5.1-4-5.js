// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5.1-4-5
description: >
    [[Call]] - length of parameters of 'target' is 0, length of
    'boundArgs' is 0, length of 'ExtraArgs' is 1, and without
    'boundThis'
---*/

var func = function() {
  return arguments[0] === 1;
};

var newFunc = Function.prototype.bind.call(func);

assert(newFunc(1), 'newFunc(1) !== true');
