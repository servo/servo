// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.7.4-2
description: >
    RegExp.prototype.multiline is an accessor property whose set
    accessor function is undefined
---*/

  var d = Object.getOwnPropertyDescriptor(RegExp.prototype, 'multiline');
  

assert.sameValue(typeof d.get, 'function', 'typeof d.get');
assert.sameValue(d.set, undefined, 'd.set');
assert.sameValue(d.enumerable, false, 'd.enumerable');
assert.sameValue(d.configurable, true, 'd.configurable');
