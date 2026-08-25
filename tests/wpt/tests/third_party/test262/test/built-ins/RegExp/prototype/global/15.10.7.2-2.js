// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.7.2-2
description: >
    RegExp.prototype.global is an accessor property whose set accessor
    function is undefined
---*/

  var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, 'global');
  

assert.sameValue(typeof desc.get, 'function', 'typeof desc.get');
assert.sameValue(desc.set, undefined, 'desc.set');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
