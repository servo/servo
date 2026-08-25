// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-10-1
description: >
    Function.prototype.bind - internal property [[Class]] of 'F' is
    set as Function
---*/

var foo = function() {};

var obj = foo.bind({});

assert.sameValue(Object.prototype.toString.call(obj), "[object Function]", 'Object.prototype.toString.call(obj)');
