// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.0_4-17gs
description: >
    Strict Mode - SourceElements is not evaluated as strict mode code
    when a Function constructor is contained in strict mode code
flags: [onlyStrict]
---*/

var _13_0_4_17_fun = new Function('eval = 42;');
