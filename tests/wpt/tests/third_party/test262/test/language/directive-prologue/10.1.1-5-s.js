// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-5-s
description: >
    Strict Mode - Use Strict Directive Prologue is ''use strict';'
    which appears at the beginning of the block
flags: [noStrict]
---*/

assert.throws(SyntaxError, function() {
        "use strict";

            eval("var public = 1;");
});
