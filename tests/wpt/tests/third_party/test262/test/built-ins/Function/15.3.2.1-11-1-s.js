// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-1-s
description: >
    Duplicate seperate parameter name in Function constructor throws
    SyntaxError in strict mode
flags: [noStrict]
---*/


assert.throws(SyntaxError, function() {
  Function('a', 'a', '"use strict";');
});
