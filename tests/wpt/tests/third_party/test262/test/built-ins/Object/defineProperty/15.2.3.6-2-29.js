// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-29
description: >
    Object.defineProperty - argument 'P' is a number that converts to
    a string (value is 1(following 20 zeros).1)
---*/

var obj = {};
Object.defineProperty(obj, 100000000000000000000.1, {});

assert(obj.hasOwnProperty("100000000000000000000"), 'obj.hasOwnProperty("100000000000000000000") !== true');
