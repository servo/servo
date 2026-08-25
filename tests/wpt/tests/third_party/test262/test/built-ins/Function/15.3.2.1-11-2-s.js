// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-2-s
description: >
    Duplicate seperate parameter name in Function constructor called
    from strict mode allowed if body not strict
flags: [onlyStrict]
---*/

Function('a', 'a', 'return;');
