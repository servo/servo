// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'O' is a Boolean object
---*/

var boolObj = new Boolean(false);
var preCheck = Object.isExtensible(boolObj);
Object.seal(boolObj);

assert(preCheck, 'preCheck !== true');
assert(Object.isSealed(boolObj), 'Object.isSealed(boolObj) !== true');
