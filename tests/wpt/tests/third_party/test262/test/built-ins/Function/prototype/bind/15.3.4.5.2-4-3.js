// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5.2-4-3
description: >
    [[Construct]] - length of parameters of 'target' is 0, length of
    'boundArgs' is 0, length of 'ExtraArgs' is 0, and without
    'boundThis'
---*/

var func = function() {
  return new Boolean(arguments.length === 0);
};

var NewFunc = Function.prototype.bind.call(func);

var newInstance = new NewFunc();

assert.sameValue(newInstance.valueOf(), true, 'newInstance.valueOf()');
