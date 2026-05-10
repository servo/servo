// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-4-s
description: >
    eval - a function expr assigning into 'eval' throws a SyntaxError
    in strict mode
flags: [onlyStrict]
---*/


assert.throws(SyntaxError, function() {
    eval('(function () { eval = 42; })()');
});
