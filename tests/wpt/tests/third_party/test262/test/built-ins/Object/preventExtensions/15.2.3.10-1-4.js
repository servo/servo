// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-1-4
description: >
    Object.preventExtensions does not throw TypeError if 'O' is a
    string primitive value
---*/

Object.preventExtensions("abc");
