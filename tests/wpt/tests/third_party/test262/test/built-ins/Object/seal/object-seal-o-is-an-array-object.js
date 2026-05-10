// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'O' is an Array object
---*/

var arr = [0, 1];
var preCheck = Object.isExtensible(arr);
Object.seal(arr);

assert(preCheck, 'preCheck !== true');
assert(Object.isSealed(arr), 'Object.isSealed(arr) !== true');
