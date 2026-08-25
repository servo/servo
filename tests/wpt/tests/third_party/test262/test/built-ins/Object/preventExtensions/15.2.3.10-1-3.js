// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-1-3
description: >
    Object.preventExtensions does not throw TypeError if 'O' is a
    boolean primitive value
---*/

Object.preventExtensions(true);
