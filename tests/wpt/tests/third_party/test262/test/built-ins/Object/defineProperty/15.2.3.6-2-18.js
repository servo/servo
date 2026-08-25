// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-18
description: >
    Object.defineProperty - argument 'P' is a number that converts to
    string (value is 1e+21)
---*/

var obj = {};
Object.defineProperty(obj, 1e+21, {});

assert(obj.hasOwnProperty("1e+21"), 'obj.hasOwnProperty("1e+21") !== true');
