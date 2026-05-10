// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.13-2-22
description: Object.isExtensible returns true if 'O' is extensible
---*/

var obj = {};

assert(Object.isExtensible(obj), 'Object.isExtensible(obj) !== true');
