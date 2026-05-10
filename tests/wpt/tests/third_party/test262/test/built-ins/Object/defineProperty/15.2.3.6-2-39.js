// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-39
description: >
    Object.defineProperty - argument 'P' is an array that converts to
    a string
---*/

var obj = {};
Object.defineProperty(obj, [1, 2], {});

assert(obj.hasOwnProperty("1,2"), 'obj.hasOwnProperty("1,2") !== true');
