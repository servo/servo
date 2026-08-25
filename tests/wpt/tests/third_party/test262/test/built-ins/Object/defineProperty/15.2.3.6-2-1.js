// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-1
description: >
    Object.defineProperty - argument 'P' is undefined that converts to
    string 'undefined'
---*/

var obj = {};
Object.defineProperty(obj, undefined, {});

assert(obj.hasOwnProperty("undefined"), 'obj.hasOwnProperty("undefined") !== true');
