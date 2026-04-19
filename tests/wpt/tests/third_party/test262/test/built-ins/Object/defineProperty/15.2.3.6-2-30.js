// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-30
description: >
    Object.defineProperty - argument 'P' is a number that converts to
    a string (value is 1(following 21 zeros).1)
---*/

var obj = {};
Object.defineProperty(obj, 1000000000000000000000.1, {});

assert(obj.hasOwnProperty("1e+21"), 'obj.hasOwnProperty("1e+21") !== true');
