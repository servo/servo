// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.2.1-11-3
description: >
    Function constructor may have a formal parameter named 'eval' if
    body is not strict mode
---*/

Function('eval', 'return;');
