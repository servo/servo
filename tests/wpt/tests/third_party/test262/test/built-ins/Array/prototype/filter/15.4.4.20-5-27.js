// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - Array.isArray(arg) returns true when arg
    is the returned array
---*/

var newArr = [11].filter(function() {});

assert(Array.isArray(newArr), 'Array.isArray(newArr) !== true');
