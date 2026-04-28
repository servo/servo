// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-1
description: >
    Duplicate separate parameter name in Function constructor allowed
    if body not strict
---*/

Function('a', 'a', 'return;');
