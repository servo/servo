// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-0-3
description: Object.getPrototypeOf must take 1 parameter
---*/


assert.throws(TypeError, function() {
  Object.getPrototypeOf();
});
