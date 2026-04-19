// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10.1-10-s
description: >
    with statement in strict mode throws SyntaxError (eval, where the
    container function is strict)
flags: [onlyStrict]
---*/

  // wrapping it in eval since this needs to be a syntax error. The
  // exception thrown must be a SyntaxError exception. Note that eval
  // inherits the strictness of its calling context.  
assert.throws(SyntaxError, function() {
    eval("\
          var o = {};\
          with (o) {}\
       ");
});
