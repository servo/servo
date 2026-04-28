// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'O' is an Arguments object
---*/

var argObj = (function() {
  return arguments;
})();

var preCheck = Object.isExtensible(argObj);
Object.seal(argObj);

assert(preCheck, 'preCheck !== true');
assert(Object.isSealed(argObj), 'Object.isSealed(argObj) !== true');
