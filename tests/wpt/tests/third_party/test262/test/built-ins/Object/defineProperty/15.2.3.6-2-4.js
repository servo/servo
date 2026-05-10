// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-4
description: >
    Object.defineProperty - argument 'P' is a boolean whose value is
    true
---*/

var obj = {};
Object.defineProperty(obj, true, {});

assert(obj.hasOwnProperty("true"), 'obj.hasOwnProperty("true") !== true');
