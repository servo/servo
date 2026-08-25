// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5.1-4-2
description: >
    [[Call]] - 'F''s [[BoundThis]] is used as the 'this' value of
    calling the [[Call]] internal method of 'F''s [[TargetFunction]]
    when 'F' is called
---*/

var obj = {
  "prop": "a"
};

var func = function() {
  return this;
};

var newFunc = Function.prototype.bind.call(func, obj);

assert.sameValue(newFunc(), obj, 'newFunc()');
