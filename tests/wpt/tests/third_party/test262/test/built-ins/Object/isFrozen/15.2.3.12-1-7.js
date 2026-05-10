// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-1-7
description: >
    Object.isFrozen applies to non-array object which contains index
    named properties
---*/

var obj = Object.freeze({
  0: 0,
  1: 1,
  1000: 1000
});

assert(Object.isFrozen(obj), 'Object.isFrozen(obj) !== true');
