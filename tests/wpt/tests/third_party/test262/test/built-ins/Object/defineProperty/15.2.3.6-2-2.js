// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-2
description: >
    Object.defineProperty - argument 'P' is null that converts to
    string 'null'
---*/

var obj = {};
Object.defineProperty(obj, null, {});

assert(obj.hasOwnProperty("null"), 'obj.hasOwnProperty("null") !== true');
