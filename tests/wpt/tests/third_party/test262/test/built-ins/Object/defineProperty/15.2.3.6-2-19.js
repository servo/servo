// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-19
description: >
    Object.defineProperty - argument 'P' is a number that converts to
    string (value is 1e+22)
---*/

var obj = {};
Object.defineProperty(obj, 1e+22, {});

assert(obj.hasOwnProperty("1e+22"), 'obj.hasOwnProperty("1e+22") !== true');
