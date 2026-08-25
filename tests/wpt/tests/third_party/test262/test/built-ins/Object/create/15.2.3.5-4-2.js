// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-2
description: Object.create - 'Properties' is undefined
---*/

var newObj = Object.create({}, undefined);

assert((newObj instanceof Object), '(newObj instanceof Object) !== true');
