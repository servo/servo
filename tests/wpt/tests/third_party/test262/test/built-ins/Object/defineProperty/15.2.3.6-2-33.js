// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-33
description: Object.defineProperty - argument 'P' is applied to an empty string
---*/

var obj = {};
Object.defineProperty(obj, "", {});

assert(obj.hasOwnProperty(""), 'obj.hasOwnProperty("") !== true');
