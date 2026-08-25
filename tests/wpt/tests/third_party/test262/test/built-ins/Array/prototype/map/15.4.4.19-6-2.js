// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - the returned array is instanceof Array
---*/

var newArr = [11].map(function() {});

assert(newArr instanceof Array, 'newArr instanceof Array !== true');
