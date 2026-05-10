// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-2-6
description: Object.keys returns the standard built-in Array that is not frozen
---*/

var o = {
  x: 1,
  y: 2
};

var a = Object.keys(o);

assert.sameValue(Object.isFrozen(a), false, 'Object.isFrozen(a)');
