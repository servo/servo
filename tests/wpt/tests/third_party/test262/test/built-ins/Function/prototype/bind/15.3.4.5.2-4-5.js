// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5.2-4-5
description: >
    [[Construct]] - length of parameters of 'target' is 0, length of
    'boundArgs' is 0, length of 'ExtraArgs' is 0, and with 'boundThis'
---*/

var obj = {
  prop: "abc"
};

Object.prototype.verifyThis = "verifyThis";
var func = function() {
  return new Boolean(arguments.length === 0 && Object.prototype.toString.call(this) === "[object Object]" &&
    this.verifyThis === "verifyThis");
};

var NewFunc = Function.prototype.bind.call(func, obj);

var newInstance = new NewFunc();

assert(newInstance.valueOf(), 'newInstance.valueOf() !== true');
