// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-d-4
description: Object.freeze - 'O' is a Boolean object
---*/

var boolObj = new Boolean(false);

Object.freeze(boolObj);

assert(Object.isFrozen(boolObj), 'Object.isFrozen(boolObj) !== true');
