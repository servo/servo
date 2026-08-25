// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-2-2
description: Object.getOwnPropertyNames - returned array is an instance of Array
---*/

var obj = {};
var result = Object.getOwnPropertyNames(obj);

assert(result instanceof Array, 'result instanceof Array !== true');
