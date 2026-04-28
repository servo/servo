// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-17
description: >
    Object.defineProperty - argument 'P' is a number that converts to
    a string (value is 1e+20)
---*/

var obj = {};
Object.defineProperty(obj, 1e+20, {});

assert(obj.hasOwnProperty("100000000000000000000"), 'obj.hasOwnProperty("100000000000000000000") !== true');
