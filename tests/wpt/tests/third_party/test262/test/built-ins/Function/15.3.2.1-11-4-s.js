// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-4-s
description: >
    Function constructor call from strict code with formal parameter
    named 'eval' does not throws SyntaxError if function body is not
    strict mode
flags: [onlyStrict]
---*/

Function('eval', 'return;');
