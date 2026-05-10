// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5.2-4-4
description: >
    [[Construct]] - length of parameters of 'target' is 0, length of
    'boundArgs' is 0, length of 'ExtraArgs' is 1, and without
    'boundThis'
---*/

var func = function() {
  return new Boolean(arguments[0] === 1 && arguments.length === 1);
};

var NewFunc = Function.prototype.bind.call(func);

var newInstance = new NewFunc(1);

assert.sameValue(newInstance.valueOf(), true, 'newInstance.valueOf()');
