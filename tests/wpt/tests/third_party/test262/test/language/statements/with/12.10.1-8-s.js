// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10.1-8-s
description: >
    with statement in strict mode throws SyntaxError (function
    expression, where the container Function is strict)
flags: [noStrict]
---*/


assert.throws(SyntaxError, function() {
    Function("\
              \'use strict\'; \
              var f1 = function () {\
                  var o = {}; \
                  with (o) {}; \
                }\
            ");
});
