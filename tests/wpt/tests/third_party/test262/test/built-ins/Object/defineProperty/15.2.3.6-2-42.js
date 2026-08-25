// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-42
description: >
    Object.defineProperty - argument 'P' is a Number Object that
    converts to a string
---*/

var obj = {};
Object.defineProperty(obj, new Number(123), {});

assert(obj.hasOwnProperty("123"), 'obj.hasOwnProperty("123") !== true');
