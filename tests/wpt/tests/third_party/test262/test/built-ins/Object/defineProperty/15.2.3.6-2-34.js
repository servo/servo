// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-34
description: Object.defineProperty - argument 'P' is applied to string 'AB  \cd'
---*/

var obj = {};
Object.defineProperty(obj, "AB\n\\cd", {});

assert(obj.hasOwnProperty("AB\n\\cd"), 'obj.hasOwnProperty("AB\n\\cd") !== true');
