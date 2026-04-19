// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-11-1
description: >
    Function.prototype.bind - internal property [[Prototype]] of 'F'
    is set as Function.prototype
---*/

var foo = function() {};

Function.prototype.property = 12;
var obj = foo.bind({});

assert.sameValue(obj.property, 12, 'obj.property');
