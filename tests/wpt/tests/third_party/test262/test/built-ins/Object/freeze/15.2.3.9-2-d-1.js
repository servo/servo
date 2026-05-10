// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-d-1
description: Object.freeze - 'O' is a Function object
---*/

var funObj = function() {};

Object.freeze(funObj);

assert(Object.isFrozen(funObj), 'Object.isFrozen(funObj) !== true');
