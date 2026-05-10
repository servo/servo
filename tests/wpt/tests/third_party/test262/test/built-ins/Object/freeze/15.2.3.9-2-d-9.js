// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-d-9
description: Object.freeze - 'O' is the Arguments object
---*/

var argObj = (function() {
  return arguments;
}());

Object.freeze(argObj);

assert(Object.isFrozen(argObj), 'Object.isFrozen(argObj) !== true');
