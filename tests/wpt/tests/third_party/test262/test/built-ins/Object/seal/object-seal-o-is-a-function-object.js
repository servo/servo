// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'O' is a Function object
---*/

var fun = function() {};
var preCheck = Object.isExtensible(fun);
Object.seal(fun);

assert(preCheck, 'preCheck !== true');
assert(Object.isSealed(fun), 'Object.isSealed(fun) !== true');
