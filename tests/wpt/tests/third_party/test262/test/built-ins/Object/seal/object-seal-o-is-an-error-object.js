// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'O' is an Error object
---*/

var errObj = new Error();
var preCheck = Object.isExtensible(errObj);
Object.seal(errObj);

assert(preCheck, 'preCheck !== true');
assert(Object.isSealed(errObj), 'Object.isSealed(errObj) !== true');
