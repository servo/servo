// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-2-s
description: >
    eval - a function assigning into 'eval' throws SyntaxError in
    strict mode
flags: [onlyStrict]
---*/


assert.throws(SyntaxError, function() {
    eval('function foo() { eval = 42; }; foo()');
});
